build:
	- ./scripts/build.sh
	- npm install --prefix ./cli
deploy:
	npm run dev --prefix ./cli
ibc-nodes:
	./scripts/ibc-setup.sh