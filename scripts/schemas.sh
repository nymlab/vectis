for dir in $PWD/contracts/*/; do
 cd $dir
 cargo run --example schema
 cd -
done