// Import the necessary classes from the '@polkadot/api' and '@polkadot/keyring' modules
const { ApiPromise } = require('@polkadot/api');
const { Keyring } = require('@polkadot/keyring');

async function main () {
  // Log the start of the execution
  console.log("Starting script...");

  // Instantiate the API
  console.log("Creating API promise...");
  const api = await ApiPromise.create();

  // Construct the keyring after the API (crypto has an async init)
  console.log("Creating keyring...");
  const keyring = new Keyring({ type: 'ethereum' });

  // Get the private key from the command line arguments
  const privateKey = process.argv[2];

  // Check that the private key was provided and is in the correct format
  if (!privateKey || !/^0x[a-fA-F0-9]{64}$/.test(privateKey)) {
    console.error('Please provide a private key as a command line argument. It should be a 66 character string, beginning with "0x" and followed by 64 hexadecimal characters.');
    process.exit(1);
  }

  const sender = keyring.addFromUri(privateKey);
  console.log(`Created account with address ${sender.address}`);

  // Get the target address from the command line arguments
  const target = process.argv[3];

  // Check that the target address was provided and is in the correct format
  if (!target || !/^0x[a-fA-F0-9]{40}$/.test(target)) {
    console.error('Please provide a target address as a command line argument. It should be a 42 character string, beginning with "0x" and followed by 40 hexadecimal characters.');
    process.exit(1);
  }

  // Get the transfer amount from the command line arguments
  const amount = process.argv[4];

  // Check that the amount was provided and is a number
  if (!amount || isNaN(amount)) {
    console.error('Please provide the transfer amount as a command line argument. It should be a number.');
    process.exit(1);
  }

  // Create a extrinsic, transferring the specified amount to the target
  const transfer = api.tx.balances.transfer(target, amount);

  // Sign and send the transaction using our account
  console.log(`Transferring ${amount} units to ${target}...`);
  const hash = await transfer.signAndSend(sender);

  console.log('Transfer sent with hash', hash.toHex());
}

// Execute the main function
main()
  .catch(error => {
    // Log any errors that occur during execution
    console.error('An error occurred:', error);
  })
  .finally(() => {
    // Exit the process once execution is complete
    console.log("Script execution complete.");
    process.exit();
  });
