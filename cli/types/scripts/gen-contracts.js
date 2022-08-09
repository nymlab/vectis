const codegen = require('@cosmwasm/ts-codegen').default;
const { join } = require('path');
const contracts = (path) => join( __dirname, '../../../contracts/', path);

codegen({
  contracts: [
    {
      name: 'Proxy',
      dir: contracts('proxy/schema')
    },
    {
      name: 'Factory',
      dir: contracts('factory/schema')
    },
    {
      name: 'Govec',
      dir: contracts('govec/schema')
    }
  ],
  outPath: join( __dirname, '../contracts'),
  options: {
    bundle: {
      bundleFile: 'index.ts',
      scope: 'contracts'
    }
  }
}).then(() => {
  console.log('✨ all done!');
});
