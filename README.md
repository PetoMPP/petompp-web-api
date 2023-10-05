# Main PetoMPP webpage backend

This repository contains the source code of the webpage API backend.
It is hosted at https://peto-main-api.azurewebsites.net/ but it's intended use is for webpage, so no documenting that.

## Used technologies

This project I decided to develop in Rust, as I did my previous API using .NET and I wanted to compare either.

The framework used to spin up the webserver is [Rocket.rs](https//rocket.rs/).
It provides a nice and clean way of defining response handlers, like in [resources.rs](/src/controllers/resources.rs)
```rust
#[get("/<key>?<lang>")]
async fn get<'a>(
    key: &'a str,
    lang: &'a str,
    pool: &dyn ResourcesRepo,
) -> Result<Json<ApiResponse<'a, String>>, ApiError<'a>> {
    Ok(Json(ApiResponse::ok(pool.get(key, lang)?)))
}
```
Of course theres a lot of code behind the scenes to make it look so clean,
as `ResourcesRepo`, `ApiResponse` and `ApiError` have to implement traits to be used like that.

As we see in previous code snippet, there's a pool of database connections injected.
For this application I went with PostgreSQL as a database, as it was easy to create in Azure,
without need for managing it in a docker image.

The library doing the hard work of talking to it is [Diesel.rs](https://diesel.rs/).
This is an ORM solution that also handles pooling and migrations,
although migration scripts are expected to be written manually (the `up.sql` and `down.sql` files).
After the migration run, the [schema.rs](/scr/schema.rs) is created/updated and later used in codebase to execute typed queries,
as in the `get` function of `ResourcesRepo` implementation in [repo.rs](/src/repositories/resources/repo.rs)
```rust
impl ResourcesRepo for PgPool {
    fn get(&self, key: &str, lang: &str) -> Result<String, Error> {
        let mut conn = self.get()?;
        let q = resources::dsl::resources.filter(resources::key.eq(key));
        let res = match lang {
            "pl" => {
                let (pl, en) = q
                    .select((resources::pl, resources::en)) // Query with fallback language as pl is not guaranteed
                    .get_result::<(Option<String>, String)>(&mut conn)?;
                pl.unwrap_or(en)
            }
            _ => q.select(resources::en).get_result::<String>(&mut conn)?,
        };
        Ok(res)
    }
  /* ... */
}
```

## Features

### Authentication

This API uses JWT authentication. The implementation of token creation and validation is fairly simple
```rust
pub fn create_token(secrets: &Secrets, user: &User) -> Result<String, AuthError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secrets.api_secret.as_bytes()).unwrap();
    let claims: BTreeMap<String, String> = Claims::try_from(user.clone())?.into();
    Ok(claims.sign_with_key(&key)?)
}

pub fn validate_token(secrets: &Secrets, token: &str) -> Result<Claims, AuthError> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(secrets.api_secret.as_bytes()).unwrap();
    let token_data: BTreeMap<String, String> = token.verify_with_key(&key)?;

    Claims::try_from(token_data)
}
```
The `Claims` part is doing the mapping and validation logic.
Those are injected to handlers as parameters and are build from request context.
Such building is done by implementing `FromRequest` trait
```rust
#[rocket::async_trait]
impl<'r> FromRequest<'r> for Claims {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, (Status, Self::Error), ()> {
        let secrets = request.rocket().state::<Secrets>().unwrap();
        let Some(token) = request.headers().get_one("Authorization") else {
            return Outcome::Failure((Status::Unauthorized, ()));
        };
        let Some(token) = token.strip_prefix("Bearer ") else {
            return Outcome::Failure((Status::Unauthorized, ()));
        };
        let Ok(claims) = validate_token(secrets, token) else {
            return Outcome::Failure((Status::Unauthorized, ()));
        };
        Outcome::Success(claims)
    }
}
```
As seen above those implementation define error responses for the request to return it without calling the actual handler.
The handler for `AdminClaims` is a wrapper around Claims with additional check for user role
```rust
pub struct AdminClaims(Claims);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminClaims {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, (Status, Self::Error), ()> {
        let Outcome::Success(claims) = request.guard::<Claims>().await else {
            return Outcome::Failure((Status::Unauthorized, ()));
        };
        if claims.acs != Role::Admin {
            return Outcome::Failure((Status::Unauthorized, ()));
        }
        Outcome::Success(Self(claims))
    }
}
```

These `Request guards` as Rocket calls them are usually discarded when injected as we don't usually work with user data in other context than `UserController`
```rust
#[put("/<key>", data = "<value>")]
async fn create<'a>(
    _admin_claims: AdminClaims,
    key: &'a str,
    value: Json<ResourceData>,
    pool: &dyn ResourcesRepo,
) -> Result<Json<ApiResponse<'a, ResourceData>>, ApiError<'a>> {
    let value = ResourceData {
        key: Some(key.to_string()),
        ..value.into_inner()
    };
    Ok(Json(ApiResponse::ok(pool.create(&value)?)))
}
```

### Azure Blob image upload

The direct upload to blob storage is handled in API instead of being done directly in the browser.
It was possible only due to existance of [azure_storage_blobs](https://github.com/Azure/azure-sdk-for-rust/) crate and its friends.
It allows to use Azure API in a very simple manner as seen below
```rust
impl AzureBlobService {
    pub fn new(secrets: AzureBlobSecrets) -> Self {
        let creds = StorageCredentials::Key(secrets.account.clone(), secrets.account_key.clone());
        let client = ClientBuilder::new(&secrets.account, creds);
        Self { secrets, client }
    }

    pub async fn upload(
        &self,
        name: String,
        folder: String,
        data: Vec<u8>,
        content_type: String,
    ) -> Result<(), Error> {
        Ok(self
            .client
            .clone()
            .blob_client(
                self.secrets.container_name.clone(),
                format!("{}/{}", folder, name),
            )
            .put_block_blob(data)
            .content_type(content_type)
            .await
            .map(|_| ())?)
    }
}
```

As always the more complicated part was the validation of files being transferred,
and it's kinda long, but pretty straightforward
```rust
#[put("/?<folder>", data = "<img>")]
async fn upload<'a>(
    _claims: Claims,
    content_type: &ContentType,
    limits: &Limits,
    blob_service: &State<AzureBlobService>,
    filename_service: &State<FilenameService>,
    folder: String,
    img: Data<'a>,
) -> Result<Json<ApiResponse<'a, String>>, ApiError<'a>> {
    if !content_type.is_jpeg() && !content_type.is_png() && !content_type.is_bmp() {
        println!("Invalid media type");
        return Err(Error::from(Status::BadRequest).into());
    }
    let Some(ext) = content_type.extension() else {
        println!("No ext");
        return Err(Error::from(Status::BadRequest).into());
    };
    if folder.is_empty() {
        println!("No folder");
        return Err(Error::from(Status::BadRequest).into());
    }
    if !filename_service.is_valid(&folder) {
        println!("Invalid folder");
        return Err(Error::from(Status::BadRequest).into());
    }
    let filename = format!("{}.{}", uuid::Uuid::new_v4(), ext);
    let data = img
        .open(limits.get("file").unwrap_or(5.mebibytes()))
        .into_bytes()
        .await
        .map_err(|_| Error::from(Status::InternalServerError))?;
    if !data.is_complete() {
        return Err(Error::from(Status::PayloadTooLarge).into());
    }
    blob_service
        .upload(
            filename.clone(),
            folder.clone(),
            data.to_vec(),
            content_type.to_string(),
        )
        .await?;
    Ok(Json(ApiResponse::ok(filename)))
}
```

## Feedback
I am very fresh in the world of the web and any feedback, issues and overall thoughts are more then welcome and I'm happy to hear them all :)
