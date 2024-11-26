import { Address, Keypair, nativeToScVal, Networks, Operation, TransactionBuilder, xdr } from "@stellar/stellar-sdk";
import { DEFAULT_TIMEOUT } from "@stellar/stellar-sdk/contract";
import { Api, assembleTransaction, Server } from "@stellar/stellar-sdk/rpc";

// const SAC = 'CDQKZ76ZS7LYDOZ2E7OG5LUJEWDDUNYBVYRJTBJK6645DZBNJWA7DXCR'
const SAC = 'CB23WRDQWGSP6YPMY4UV5C4OW5CBTXKYN3XEATG7KJEZCXMJBYEHOUOV'

// const contractID = 'CDBG4XY2T5RRPH7HKGZIWMR2MFPLC6RJ453ITXQGNQXG6LNVL4375MRJ'
const contractID = 'CDL74RF5BLYR2YBLCCI7F5FB6TPSCLKEJUBSD2RSVWZ4YHF3VMFAIGWA'

// const networkPassphrase = Networks.TESTNET;
const networkPassphrase = Networks.PUBLIC;
const amount = 65423522567;

// const rpc = new Server("https://soroban-testnet.stellar.org");
const rpc = new Server("https://mainnet.sorobanrpc.com");

// const keypair = Keypair.fromSecret('SA7E3D73763HEN2GNHOLNJUH3EQWLN34NRRSEKFJFSU7ENJICRGL35F6');
const keypair = Keypair.fromSecret('<mainnet-kale-admin-secret>');
const pubkey = keypair.publicKey(); // GCCX6ZAVF63XCMDFYAT6TPRUWNF3FS43YI6FOJ3JS4MWCYP4QYYJISCV

const acct = await rpc.getAccount(pubkey)
const tx = new TransactionBuilder(acct, {
    fee: (100_000).toString(),
    networkPassphrase
})
.addOperation(Operation.invokeContractFunction({
    contract: SAC,
    function: 'clawback',
    args: [
        Address.fromString(SAC).toScVal(),
        nativeToScVal(amount, { type: 'i128' })
    ]
}))
.setTimeout(0)
.build();

const simBefore = await rpc.simulateTransaction(tx);

if (
    Api.isSimulationError(simBefore)
    || !simBefore.result
    || !simBefore.result.auth
) {
    console.log(simBefore);
} else {
    const entry = xdr.SorobanAuthorizationEntry.fromXDR(simBefore.result.auth[0].toXDR());
    const credentials = entry.credentials().address();
    const lastLedger = await rpc.getLatestLedger().then(({ sequence }) => sequence);

    credentials.signatureExpirationLedger(lastLedger + DEFAULT_TIMEOUT);
    credentials.signature(xdr.ScVal.scvVec([]));

    const op = tx.operations[0] as Operation.InvokeHostFunction;

    op.auth?.splice(0, 1, entry);

    const self_invocation = new xdr.InvokeContractArgs({
        contractAddress: Address.fromString(contractID).toScAddress(),
        functionName: "__check_auth",
        args: [],
    });

    const self_entry = new xdr.SorobanAuthorizationEntry({
        credentials: xdr.SorobanCredentials.sorobanCredentialsSourceAccount(),
        rootInvocation: new xdr.SorobanAuthorizedInvocation({
            function: xdr.SorobanAuthorizedFunction.sorobanAuthorizedFunctionTypeContractFn(self_invocation),
            subInvocations: [],
        }),
    })

    op.auth?.push(self_entry)

    const simAfter = await rpc.simulateTransaction(tx);

    const txAssem = assembleTransaction(tx, simAfter).build();

    txAssem.sign(keypair);

    const sendRes = await rpc.sendTransaction(txAssem);
    const pollRes = await rpc.pollTransaction(sendRes.hash);

    if (pollRes.status === 'SUCCESS') {
        console.log(pollRes.status, pollRes.txHash);
    } else if  (pollRes.status === 'NOT_FOUND') {
        console.log(pollRes);
    } else {
        console.log(pollRes.envelopeXdr.toXDR('base64'));
        console.log('\n');
        console.log(pollRes.resultXdr.toXDR('base64'));
        console.log('\n');
        console.log(pollRes.resultMetaXdr.toXDR('base64'));
    }
}
