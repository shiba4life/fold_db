// Key Management Tab wrapper component

import KeyGenerationComponent from '../KeyGenerationComponent';

const KeyManagementTab = ({ onResult }) => {
  // For now, this just wraps the KeyGenerationComponent
  // In the future, it could include other key management features
  return (
    <div className="space-y-6">
      <KeyGenerationComponent onResult={onResult} />
    </div>
  );
};

export default KeyManagementTab;