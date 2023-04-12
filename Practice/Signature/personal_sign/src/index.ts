import { Wallet } from './wallet';

async function runProc() {

  const wallet = new Wallet("ed25519:2CH8v4LQD2o4RDoMx1ZXuo6mJCPNbDCyXHMhirLdotMW714ZxuWxjxxtZ6tMRmarudrNMjR18jQofp5F8obSMWwc", 
  "dev-1679374959912-95659865680232");
  const { accountId, publicKey, signature } = await wallet.signMessage({ 
    message:"bye",
    recipient:"giparktest.testnet",
    nonce:Buffer.from(Array.from(Array(32).keys())),
    callbackUrl:""});
    console.log(`accountId : ${accountId}`)
    console.log(`publicKey : ${publicKey}`)
    console.log(`signature : ${signature}`)
}

runProc()