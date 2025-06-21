import { useEffect } from 'react';

const LOGOUT_EVENT = 'logout';
const SESSION_EXPIRED_EVENT = 'session-expired';

export function useKeyLifecycle(clearKeys: () => void) {
  useEffect(() => {
    const handleCleanup = () => {
      clearKeys();
    };

    window.addEventListener('beforeunload', handleCleanup);
    window.addEventListener(LOGOUT_EVENT, handleCleanup);
    window.addEventListener(SESSION_EXPIRED_EVENT, handleCleanup);

    return () => {
      window.removeEventListener('beforeunload', handleCleanup);
      window.removeEventListener(LOGOUT_EVENT, handleCleanup);
      window.removeEventListener(SESSION_EXPIRED_EVENT, handleCleanup);
    };
  }, [clearKeys]);
}
