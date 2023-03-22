import { Wallet, Mail, Person } from './wallet';

async function runProc() {

  const wallet = new Wallet("ed25519:2CH8v4LQD2o4RDoMx1ZXuo6mJCPNbDCyXHMhirLdotMW714ZxuWxjxxtZ6tMRmarudrNMjR18jQofp5F8obSMWwc", 
  "dev-1679374959912-95659865680232");
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
    console.log(signature)
}

runProc()