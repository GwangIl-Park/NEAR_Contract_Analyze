import { Wallet, Mail, Person } from './wallet';

async function runProc() {

  const wallet = new Wallet("ed25519:3UTtEW3K3vpgU13iriTKw62mQPa23Jq55NLw2ue4VMoVwfVfPiuepYCr8PxA2aUR7oCKnNrUzD5nYtsnGGTz4Y7n", 
  "dev-1679486636127-69282739867644");
  let from :Person = {
    name:"gipark",
    wallet:"giparktest.testnet"
  }
  let to:Person = {
    name:"gipark2",
    wallet:"gipark2.testnet"
  }
  let message:Mail = {
    from,
    to,
    contents:"Hi"
  }
  const { accountId, publicKey, signature } = await wallet.signMessage({ 
    message,
    recipient:"giparktest.testnet",
    nonce:Buffer.from(Array.from(Array(32).keys())),
    callbackUrl:""});
    console.log(`accountId : ${accountId}`)
    console.log(`publicKey : ${publicKey}`)
    console.log(`signature : ${signature}`)
}

runProc()