// Script to check if the FoldDB provider is working
console.log('Checking FoldDB provider status...');
if (window.foldDB) {
  console.log('FoldDB provider exists:', window.foldDB);
  
  // Dispatch a connect event to ensure the app knows we're connected
  if (window.foldDB.dispatchEvent) {
    console.log('Dispatching connect event');
    window.foldDB.dispatchEvent('connect', { isConnected: true });
  } else {
    console.log('dispatchEvent method not found on foldDB');
  }
  
  // Test the getPublicKey method
  if (window.foldDB.getPublicKey) {
    console.log('Testing getPublicKey method...');
    window.foldDB.getPublicKey()
      .then(publicKey => {
        console.log('Public key retrieved successfully:', publicKey);
      })
      .catch(error => {
        console.error('Error getting public key:', error);
      });
  } else {
    console.log('getPublicKey method not found on foldDB');
  }
} else {
  console.log('FoldDB provider does not exist');
}
