-- This file should undo anything in `up.sql`
DELETE FROM resources WHERE key = 'blog-intro' OR key = 'editor-intro';
