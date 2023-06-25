// Import the necessary classes from the '@polkadot/api' module
const { ApiPromise, WsProvider } = require('@polkadot/api');

// Define the main function that will run our script
async function main() {
  // Log the start of the execution
  console.log("Starting script...");

  // Get the address from the command line arguments
  const address = process.argv[2];

  // Check that the address was provided
  if (!address) {
    console.error('Please provide an address as a command line argument.');
    process.exit(1);
  }

  // Check that the address is in the correct format
  if (!/^0x[a-fA-F0-9]{40}$/.test(address)) {
    console.error('The address is not in the correct format. It should be a 42 character string, beginning with "0x" and followed by 40 hexadecimal characters.');
    process.exit(1);
  }

  // Create a WebSocket provider
  console.log("Creating WebSocket provider...");
  const wsProvider = new WsProvider('ws://127.0.0.1:9944');

  // Create the API promise
  console.log("Creating API promise...");
  const api = await ApiPromise.create({ provider: wsProvider });

  // Log the address we're checking
  console.log(`Checking balance for address: ${address}`);

  // Fetch the balance for the given address
  const { data: balance } = await api.query.system.account(address);

  // Log the balances
  console.log(`Free balance: ${balance.free}`);
  console.log(`Reserved balance: ${balance.reserved}`);
  console.log(`Total balance: ${balance.free.add(balance.reserved)}`);
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
