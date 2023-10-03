-- Your SQL goes here
CREATE TABLE resources (
    key VARCHAR(64) PRIMARY KEY,
    en TEXT NOT NULL,
    pl TEXT NULL
);

INSERT INTO resources (key, en, pl) VALUES
('home-content', '*TBD*', NULL),
('about-content', '*TBD*', NULL),
('contact-content', '*TBD*', NULL),
('admin-panel-content', '*TBD*', NULL);
