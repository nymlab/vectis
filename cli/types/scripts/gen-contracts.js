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
    },
    {
      name: "DaoTunnel",
      dir: contracts('dao_tunnel/schema')
    },
    {
      name: "RemoteTunnel",
      dir: contracts('remote_tunnel/schema')
    }
  ],
  outPath: join( __dirname, '../contracts'),
  options: {
    bundle: {
      enabled: false,
    }
  }
}).then(() => {
  console.log('✨ all done!');
});
