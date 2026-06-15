import { useSyncExternalStore } from 'react';
import { getAppStoreSnapshot, subscribeAppStore } from '$lib/stores/app';

export function useAppStore() {
  return useSyncExternalStore(subscribeAppStore, getAppStoreSnapshot, getAppStoreSnapshot);
}
