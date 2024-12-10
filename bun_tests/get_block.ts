import { scValToNative, xdr } from "@stellar/stellar-sdk";
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

await getFarmBlock()
await getBlock(Number(values.block))