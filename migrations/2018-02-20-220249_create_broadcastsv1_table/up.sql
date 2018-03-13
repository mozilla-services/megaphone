CREATE TABLE broadcastsv1 (
    broadcaster_id VARCHAR(64) NOT NULL,
    bchannel_id VARCHAR(128) NOT NULL,
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP NOT NULL,
    version VARCHAR(200) NOT NULL,
    PRIMARY KEY(broadcaster_id, bchannel_id)
);
