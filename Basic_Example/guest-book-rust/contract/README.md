# Guest Book Contract

The smart contract stores messages from users. Messages can be `premium` if the user attaches sufficient money (0.1 $NEAR).

```rust
// Public - Adds a new message.
#[payable]
pub fn add_message(&mut self, text: String) {
  // If the user attaches more than 0.01N the message is premium
  let premium = env::attached_deposit() >= POINT_ONE;
  let sender = env::predecessor_account_id();

  let message = PostedMessage{premium, sender, text};
  self.messages.push(&message);
}

// Returns an array of messages.
pub fn get_messages(&self, from_index:Option<U128>, limit:Option<u64>) -> Vec<PostedMessage>{
  let from = u128::from(from_index.unwrap_or(U128(0)));

  self.messages.iter()
  .skip(from as usize)
  .take(limit.unwrap_or(10) as usize)
  .collect()
}
```

<br />

# Quickstart

1. Make sure you have installed [rust](https://rust.org/).
2. Install the [`NEAR CLI`](https://github.com/near/near-cli#setup)

<br />

## 1. Build and Deploy the Contract
You can automatically compile and deploy the contract in the NEAR testnet by running:

```bash
./deploy.sh
```

Once finished, check the `neardev/dev-account` file to find the address in which the contract was deployed:

```bash
cat ./neardev/dev-account
# e.g. dev-1659899566943-21539992274727
```

<br />

## 2. Retrieve the Stored Messages
`get_messages` is a read-only method (`view` method) that returns a slice of the vector `messages`.

`View` methods can be called for **free** by anyone, even people **without a NEAR account**!

```bash
near view <dev-account> get_messages '{"from_index":0, "limit":10}'
```

<br />

## 3. Add a Message
`add_message` adds a message to the vector of `messages` and marks it as premium if the user attached more than `0.1 NEAR`.

`add_message` is a payable method for which can only be invoked using a NEAR account. The account needs to attach money and pay GAS for the transaction.

```bash
# Use near-cli to donate 1 NEAR
near call <dev-account> add_message '{"text": "a message"}' --amount 0.1 --accountId <account>
```

**Tip:** If you would like to add a message using your own account, first login into NEAR using:

```bash
# Use near-cli to login your NEAR account
near login
```

and then use the logged account to sign the transaction: `--accountId <your-account>`.