DIR = $(notdir $(shell pwd))

build:
	cargo build -r --target wasm32-wasi
	docker compose build

run: build
	docker compose up -d

logs:
	docker compose logs app

clean:
	cargo clean
	docker compose down
	docker volume rm ${DIR}_db-data

tools:
	cargo install sqlx-cli
