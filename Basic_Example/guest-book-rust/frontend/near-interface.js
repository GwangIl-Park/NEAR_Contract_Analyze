/* Talking with a contract often involves transforming data, we recommend you to encapsulate that logic into a class */

import { utils } from 'near-api-js';

export class GuestBook {

  constructor({ contractId, walletToUse }) {
    this.contractId = contractId;
    this.wallet = walletToUse
  }

  async getMessages() {
    // Check how many messages there are and ask for the last 10
    //const total_messages = this.wallet.viewMethod({ contractId: this.contractId, method: "last_messages" })
    // const from = total_messages > 10 ? total_messages - 10 : 0;
    const from = 9;
    const messages = await this.wallet.viewMethod({ contractId: this.contractId, method: "get_messages", from })
    return messages
  }

  async addMessage(message, donation) {
    const deposit = utils.format.parseNearAmount(donation);
    return await this.wallet.callMethod({ contractId: this.contractId, method: "add_message", args: { text: message }, deposit });
  }

}