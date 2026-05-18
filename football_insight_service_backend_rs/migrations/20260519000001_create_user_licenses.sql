CREATE TABLE IF NOT EXISTS f_i_user_licenses (
    id          BIGSERIAL PRIMARY KEY,
    user_id     BIGINT NOT NULL REFERENCES f_i_users(id) ON DELETE CASCADE,
    license_key VARCHAR(16) NOT NULL UNIQUE,
    used_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_f_i_user_licenses_user_id ON f_i_user_licenses(user_id);
CREATE INDEX idx_f_i_user_licenses_license_key ON f_i_user_licenses(license_key) WHERE used_at IS NULL;
