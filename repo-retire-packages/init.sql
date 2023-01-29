CREATE TABLE IF NOT EXISTS `packages` (
    package TEXT NOT NULL,
    sha256 TEXT NOT NULL PRIMARY KEY, -- sha256sum is most likely to be unique
    size INTEGER NOT NULL,
    filename TEXT NOT NULL UNIQUE,
    version TEXT NOT NULL,
    retire_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS `package_version` ON `packages` (package, version);
