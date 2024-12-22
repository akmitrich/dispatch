create table if not exists users (
	username VARCHAR(8) primary key not null,
	password VARCHAR(64) not null,
	created_at TIMESTAMP default current_timestamp
)