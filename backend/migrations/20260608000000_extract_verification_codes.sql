CREATE TABLE verification_codes (
    id SERIAL PRIMARY KEY,
    email VARCHAR(254) NOT NULL,
    code VARCHAR(6) NOT NULL,
    purpose VARCHAR(20) NOT NULL DEFAULT 'email_verify',
    expires_at TIMESTAMPTZ NOT NULL,
    resend_count INTEGER NOT NULL DEFAULT 0,
    last_resend_at TIMESTAMPTZ
);
CREATE UNIQUE INDEX idx_verification_codes_email_purpose ON verification_codes(email, purpose);

ALTER TABLE users DROP COLUMN IF EXISTS email_verify_token;
ALTER TABLE users DROP COLUMN IF EXISTS token_expires_at;
ALTER TABLE users DROP COLUMN IF EXISTS resend_count;
ALTER TABLE users DROP COLUMN IF EXISTS last_resend_at;
