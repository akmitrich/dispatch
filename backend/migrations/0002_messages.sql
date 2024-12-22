create table if not exists messages (
	id SERIAL primary key,
	"from" VARCHAR(8) not null,
	"body" text not null,
	"timestamp" BIGINT not null
)