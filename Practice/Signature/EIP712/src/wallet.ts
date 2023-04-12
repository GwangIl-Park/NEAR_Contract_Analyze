import { KeyPair } from "near-api-js";
import * as Borsh from '@dao-xyz/borsh';
import { field, option, fixedArray } from '@dao-xyz/borsh';
const js_sha256 = require("js-sha256")

const EIP712DOMAIN_TYPEHASH = [
  198, 59, 48, 83, 117, 35, 182, 76, 53, 33, 226, 87, 66, 137, 235, 230, 230, 134, 35, 176, 229,
    204, 101, 238, 119, 187, 26, 87, 155, 227, 67, 239,];

const PERSON_TYPEHASH = [
  11, 175, 250, 237, 110, 96, 120, 229, 28, 102, 40, 124, 59, 106, 84, 249, 87, 238, 156, 1, 219,
  126, 140, 71, 170, 107, 179, 159, 22, 4, 138, 213,
];

const MAIL_TYPEHASH = [
  123, 96, 125, 176, 215, 143, 52, 177, 90, 173, 182, 32, 26, 61, 220, 53, 155, 135, 6, 4, 126,
  196, 195, 186, 217, 183, 8, 200, 246, 202, 98, 175,
];

const DOMAIN_SEPARATOR = [
  236, 186, 149, 206, 28, 208, 103, 29, 234, 94, 209, 25, 153, 238, 4, 68, 253, 236, 78, 68, 86,
  169, 6, 113, 51, 216, 182, 56, 114, 196, 230, 89,
];

interface EIP712Domain {
  name:string,
  version:string,
  chain_id:string,
  verifying_contract:string
}

export interface Person {
  name:string,
  wallet:string
}

export interface Mail {
  from:Person,
  to:Person,
  contents:string
}

interface SignMessageParams {
  message: Mail; // The message that wants to be transmitted.
  recipient: string; // The recipient to whom the message is destined (e.g. "alice.near" or "myapp.com").
  nonce: Buffer; // A nonce that uniquely identifies this instance of the message, denoted as a 32 bytes array (a fixed `Buffer` in JS/TS).
  callbackUrl?: string; // Optional, applicable to browser wallets (e.g. MyNearWallet). The URL to call after the signing process. Defaults to `window.location.href`.
}

class Payload {
  @field({ type: 'u32' })
  tag: number; // Always the same tag: 2**31 + 413
  @field({ type: 'string' })
  message: string; // The same message passed in `SignMessageParams.message`
  @field({ type: fixedArray('u8', 32) })
  nonce: number[]; // The same nonce passed in `SignMessageParams.nonce`
  @field({ type: 'string' })
  recipient: string; // The same recipient passed in `SignMessageParams.recipient`
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

class personPayload {
  @field({ type: fixedArray('u8', 32) })
  typehash:number[];
  @field({type:'string'})
  name:string;
  @field({type:'string'})
  wallet:string;
  constructor({ name, wallet }: personPayload) {
    this.typehash=PERSON_TYPEHASH;
    this.name=name;
    this.wallet=wallet;
  }
}

class mailPayload {
  @field({ type: fixedArray('u8', 32) })
  typehash:number[];
  @field({type:'string'})
  hashFrom:string;
  @field({type:'string'})
  hashTo:string;
  @field({type:'string'})
  hashContents:string;
  constructor({ hashFrom, hashTo, hashContents }: mailPayload) {
    this.typehash=MAIL_TYPEHASH;
    this.hashFrom=hashFrom;
    this.hashTo=hashTo;
    this.hashContents=hashContents;
  }
}

class messagePayload {
  @field({ type: fixedArray('u8', 32) })
  typehash:number[];
  @field({type:'string'})
  hashMail:string;
  constructor({ hashMail }: messagePayload) {
    this.typehash=DOMAIN_SEPARATOR;
    this.hashMail=hashMail
  }
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

  hashPerson(person:Person) {
    const hashedName = js_sha256.sha256.hex(person.name)
    const hashedWallet = js_sha256.sha256.hex(person.wallet)
    const payload = new personPayload({typehash:PERSON_TYPEHASH, name:hashedName, wallet:hashedWallet});
    const borshPayload = Borsh.serialize(payload);
    console.log(console.log(JSON.stringify(borshPayload)))
    
    console.log(js_sha256.sha256.array(borshPayload))
    return js_sha256.sha256.hex(borshPayload);
  }

  hashMail(mail:Mail) {
    const hashContents = js_sha256.sha256.hex(mail.contents)
    const payload = new mailPayload({typehash:MAIL_TYPEHASH, hashFrom:this.hashPerson(mail.from), hashTo:this.hashPerson(mail.to), hashContents})
    const borshPayload = Borsh.serialize(payload);
    return js_sha256.sha256.hex(borshPayload);
  }

  hashMessage(message:Mail) {
    const payload = new messagePayload({typehash:DOMAIN_SEPARATOR, hashMail:this.hashMail(message)})
    const borshPayload = Borsh.serialize(payload);
    return js_sha256.sha256.hex(borshPayload);
  }

  printPayload(payload:Payload, message:Mail) {
    console.log('---- payload ----')
    console.log(`tag : ${payload.tag}`)
    console.log(`message`)
    console.log(message)
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

    let hashedMessage:string = this.hashMessage(message)

    // Create the payload and sign it
    const payload = new Payload({ tag: 2147484061, message:hashedMessage, nonce: Array.from(nonce), recipient, callbackUrl });

    this.printPayload(payload, message)

    const borshPayload = Borsh.serialize(payload);
    const hashedPayload = js_sha256.sha256.array(borshPayload)
    console.log(`message Hash\n${hashedPayload}`)
    const { signature } = Key.sign(Uint8Array.from(hashedPayload))

    const encoded: string = Buffer.from(signature).toString('base64')
    return { accountId: this.accountId, publicKey: this.keyPair.getPublicKey().toString(), signature: encoded }
  }
}