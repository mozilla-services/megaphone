INSERT INTO broadcastsv1 (broadcaster_id, bchannel_id, version)
VALUES (?, ?, ?)
ON DUPLICATE KEY UPDATE version = ?;
