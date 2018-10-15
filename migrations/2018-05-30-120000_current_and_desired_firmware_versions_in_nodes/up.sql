alter table nodes add column desired_firmware_type integer;
alter table nodes add column desired_firmware_version integer;
CREATE TABLE sensors (
    node_id                 INTEGER,
    child_sensor_id         INTEGER ,
    sensor_type             VARCHAR,
    description             VARCHAR,
  PRIMARY KEY(node_id, child_sensor_id)
);