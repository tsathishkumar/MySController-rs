CREATE TABLE firmwares (
  firmware_type    INTEGER,
  firmware_version INTEGER,
  name             VARCHAR,
  blocks           INTEGER,
  crc              INTEGER,
  data             BLOB,
  PRIMARY KEY (firmware_type, firmware_version)
);