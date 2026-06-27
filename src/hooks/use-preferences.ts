import { useEffect, useRef, useState } from "react";
import {
  type AppSnapshotDto,
  getAppSnapshot,
  readErrorMessage,
  updatePreferences,
} from "@/lib/muc";

type Preferences = AppSnapshotDto["preferences"];

export function usePreferences(active = true) {
  const [preferences, setPreferences] = useState<Preferences | null>(null);
  const [errorText, setErrorText] = useState("");
  const preferencesRef = useRef<Preferences | null>(null);
  const saveVersionRef = useRef(0);

  useEffect(() => {
    if (!active) return;

    let disposed = false;

    async function load() {
      setErrorText("");
      try {
        const snapshot = await getAppSnapshot();
        if (!disposed) {
          preferencesRef.current = snapshot.preferences;
          setPreferences(snapshot.preferences);
        }
      } catch (error) {
        if (!disposed) {
          setErrorText(readErrorMessage(error));
        }
      }
    }

    void load();
    return () => {
      disposed = true;
    };
  }, [active]);

  function togglePreference(key: keyof Preferences) {
    const current = preferencesRef.current;
    if (!current) {
      return;
    }

    const previous = current;
    const next = {
      ...current,
      [key]: !current[key],
    };
    const saveVersion = saveVersionRef.current + 1;

    preferencesRef.current = next;
    saveVersionRef.current = saveVersion;
    setPreferences(next);
    setErrorText("");

    void updatePreferences(next)
      .then((snapshot) => {
        if (saveVersionRef.current !== saveVersion) {
          return;
        }
        preferencesRef.current = snapshot.preferences;
        setPreferences(snapshot.preferences);
      })
      .catch((error) => {
        if (saveVersionRef.current !== saveVersion) {
          return;
        }
        preferencesRef.current = previous;
        setPreferences(previous);
        setErrorText(readErrorMessage(error));
      });
  }

  return {
    preferences,
    errorText,
    togglePreference,
  };
}
