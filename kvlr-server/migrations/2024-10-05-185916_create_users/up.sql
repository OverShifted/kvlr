-- Your SQL goes here
CREATE TABLE IF NOT EXISTS public.users
(
    username character varying COLLATE pg_catalog."default" NOT NULL,
    secret_sha256 character varying COLLATE pg_catalog."default" NOT NULL,
    CONSTRAINT "PK" PRIMARY KEY (username)
)
