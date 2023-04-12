import { KeyPair } from "near-api-js";
import * as Borsh from '@dao-xyz/borsh';
import { field, option, fixedArray } from '@dao-xyz/borsh';
const js_sha256 = require("js-sha256")

interface SignMessageParams {
  message: string; // The message that wants to be transmitted.
  recipient: string; // The recipient to whom the message is destined (e.g. "alice.near" or "myapp.com").
  nonce: Buffer; // A nonce that uniquely identifies this instance of the message, denoted as a 32 bytes array (a fixed `Buffer` in JS/TS).
  callbackUrl?: string; // Optional, applicable to browser wallets (e.g. MyNearWallet). The URL to call after the signing process. Defaults to `window.location.href`.
}

class Payload {
  @field({ type: 'u32' })  //해당 속성이 어떤 형식으로 정의되는지 명시하는 decorator
  tag: number;
  @field({ type: 'string' })
  message: string;
  @field({ type: fixedArray('u8', 32) })
  nonce: number[];
  @field({ type: 'string' })
  recipient: string;
  @field({ type: option('string') })
  callbackUrl?: string;

  constructor({ message, nonce, recipient, callbackUrl }: Payload) {
    this.tag = 2147484061;
    this.message = message;
    this.nonce = nonce;
    this.recipient = recipient;
    if (callbackUrl) {
      this.callbackUrl = callbackUrl;
    }
  }
}

interface aa {
  name:string,
  age:string
}

interface AuthenticationToken {
  accountId: string; // The account name as plain text (e.g. "alice.near")
  publicKey: string; // The public counterpart of the key used to sign, expressed as a string with format "<key-type>:<base-64-key-bytes>"
  signature: string; // The base64 representation of the signature.
}
export class Wallet {
  readonly keyPair: KeyPair;
  readonly accountId: string;

  constructor(key:string, accountId:string) {
    this.keyPair = KeyPair.fromString(key);
    this.accountId = accountId;
  }

  printPayload(payload:Payload) {
    console.log('---- payload ----')
    console.log(`tag : ${payload.tag}`)
    console.log(`message : ${payload.message}`)
    console.log(`nonce : ${payload.nonce}`)
    console.log(`recipient : ${payload.recipient}`)
    console.log(`callbackUrl : ${payload.callbackUrl}`)
    console.log('----------------\n')
  }

  async signMessage({ message, recipient, nonce, callbackUrl }: SignMessageParams): Promise<AuthenticationToken> {
    // Get key from the wallet
    const Key = this.keyPair;
    // Check the nonce is a 32bytes array
    if (nonce.byteLength != 32) { throw Error("Expected nonce to be a 32 bytes buffer") }

    // Create the payload and sign it
    const payload = new Payload({ tag: 2147484061, message, nonce: Array.from(nonce), recipient, callbackUrl });
    this.printPayload(payload)
    const borshPayload = Borsh.serialize(payload);
    const hashedPayload = js_sha256.sha256.array(borshPayload)
    console.log(`message Hash\n${hashedPayload}`)
    const { signature } = Key.sign(Uint8Array.from(hashedPayload))

    const encoded: string = Buffer.from(signature).toString('base64')
    return { accountId: this.accountId, publicKey: this.keyPair.getPublicKey().toString(), signature: encoded }
  }
}