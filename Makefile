build:
	- ./scripts/build.sh
	- npm install --prefix ./cli
schemas:
	./scripts/schemas.sh && rm -rf ./contracts/*/*/schema/raw/
types:
	npm run types:generate --prefix ./cli
upload:
	npm run upload --prefix ./cli 
deploy:
	npm run deploy:hub --prefix ./cli 
nodes-setup:
	./scripts/nodes-setup.sh
