build:
	- ./scripts/build.sh
	- npm install --prefix ./cli
schemas:
	./scripts/schemas.sh
deploy:
	npm run dev --prefix ./cli
nodes-setup:
	./scripts/nodes-setup.sh