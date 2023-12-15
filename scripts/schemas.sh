for dir in $PWD/contracts/core/*/; do
 cd $dir
 cargo run
 cd -
done

for dir in $PWD/contracts/authenticators/*/; do
 cd $dir
 cargo run
 cd -
done

cd ts
npm i
npm run generate
cd ..
