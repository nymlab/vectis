build:
	- ./scripts/build.sh
	- npm install --prefix ./cli
schemas:
	./scripts/schemas.sh
types:
	npm run types:generate --prefix ./cli
deploy:
	npm run dev --prefix ./cli
nodes-setup:
	./scripts/nodes-setup.sh