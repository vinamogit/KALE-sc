import { Address, scValToNative, xdr } from "@stellar/stellar-sdk";
import { Durability, Server } from "@stellar/stellar-sdk/rpc";
import { parseArgs } from "util";

const { values } = parseArgs({
  args: Bun.argv,
  options: {
    block: {
      type: 'string',
    },
  },
  strict: true,
  allowPositionals: true,
});

const rpc = new Server("https://mainnet.sorobanrpc.com");
const CONTRACT_ID = "CDL74RF5BLYR2YBLCCI7F5FB6TPSCLKEJUBSD2RSVWZ4YHF3VMFAIGWA"
const FARMER_PK = "GBIIUZH63Z262QXGKJIP3ZU5DS7L4L2TBTYGPXRIGQXZAF25A72YNULL"

export async function getFarmBlock() {
  let block

  await rpc.getContractData(
    CONTRACT_ID,
    xdr.ScVal.scvLedgerKeyContractInstance()
  ).then(({ val }) =>
      val.contractData()
          .val()
          .instance()
          .storage()
  ).then((storage) => {
      return storage?.map((entry) => {
          const key: string = scValToNative(entry.key())[0]

          if (key === 'FarmBlock') {
              block = scValToNative(entry.val())
          }
      })
  })

  console.log('FarmBlock', block);
}

async function getBlock(index: number) {
    let block;

    await rpc.getContractData(CONTRACT_ID, xdr.ScVal.scvVec([
        xdr.ScVal.scvSymbol('Block'),
        xdr.ScVal.scvU32(Number(index))
    ]), Durability.Temporary)
        .then(({ val }) => {
            block = scValToNative(val.contractData().val())
        })

    console.log('Block', block);
}

async function getPail(index: number) {
  let pail;

  await rpc.getContractData(CONTRACT_ID, xdr.ScVal.scvVec([
      xdr.ScVal.scvSymbol('Pail'),
      Address.fromString(FARMER_PK).toScVal(),
      xdr.ScVal.scvU32(Number(index))
  ]), Durability.Temporary)
      .then(({ val }) => {
          pail = scValToNative(val.contractData().val())
      })

  console.log('Pail', pail);
}

await getFarmBlock()
await getBlock(Number(values.block))
await getPail(Number(values.block))

// https://stellar.expert/explorer/public/tx/243d4650847fb3544311be1117970cb79367b99c571e90c9d187a7533f5c6a27