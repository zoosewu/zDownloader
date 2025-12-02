// useClipboard hook - Monitor clipboard for download URLs and auto-navigate
import { useState, useEffect, useRef } from 'react';
import { invoke } from '../utils/tauri';
import { useAtomValue } from 'jotai';
import { settingsAtom } from '../stores/settingsStore';
import { useNavigation } from './useNavigation';
import { urlDetector } from '../utils/urlDetector';

interface UrlDetectionResult {
  url: string;
  isValid: boolean;
  detectorName: string;
  detectionMode: string;
  suggestedFilename?: string;
  fileExtension?: string;
}

export function useClipboard(tauriReady: boolean = false) {
  const [detectedUrl, setDetectedUrl] = useState<string | null>(null);
  const lastContentRef = useRef<string>('');
  const processingRef = useRef<boolean>(false);
  const { navigateTo } = useNavigation();
  const settings = useAtomValue(settingsAtom);

  useEffect(() => {
    console.log('[useClipboard] Tauri ready:', tauriReady);
    console.log('[useClipboard] Clipboard monitoring enabled:', settings.enableClipboardMonitoring);

    if (!tauriReady) {
      console.log('[useClipboard] Tauri not ready yet, skipping clipboard monitoring');
      return;
    }

    if (!settings.enableClipboardMonitoring) {
      console.log('[useClipboard] Clipboard monitoring is disabled, skipping');
      return;
    }

    console.log('[useClipboard] Starting clipboard polling with 400ms interval');
    const interval = setInterval(async () => {
      // Skip if still processing previous content
      if (processingRef.current) return;

      try {
        const content = await invoke<string>('get_clipboard_content');

        if (content !== lastContentRef.current) {
          processingRef.current = true;
          lastContentRef.current = content;
          console.log('[useClipboard] New clipboard content detected:', content.substring(0, 100));

          // Quick client-side validation using urlDetector
          if (!urlDetector.isValidUrl(content)) {
            console.log('[useClipboard] Not a valid URL, skipping');
            processingRef.current = false;
            return;
          }

          // Pre-filter: only check URLs with common file extensions
          // This reduces unnecessary backend calls
          if (!urlDetector.hasCommonFileExtension(content)) {
            console.log('[useClipboard] No common file extension found, skipping');
            processingRef.current = false;
            return;
          }

          console.log('[useClipboard] Calling backend detect_url...');
          try {
            const result = await invoke<UrlDetectionResult>('detect_url', { url: content });
            console.log('[useClipboard] Detection result:', result);

            if (result.isValid) {
              console.log('[useClipboard] Valid download URL detected, navigating to add-download page');
              setDetectedUrl(content);
              navigateTo('add-download', { url: content });
            } else {
              console.log('[useClipboard] URL detected but not valid for download');
            }
          } catch (e) {
            console.log('[useClipboard] URL detection failed:', e);
          } finally {
            processingRef.current = false;
          }
        }
      } catch (e) {
        console.error('[useClipboard] Failed to read clipboard:', e);
        processingRef.current = false;
      }
    }, 400); // 400ms polling interval

    return () => clearInterval(interval);
  }, [tauriReady, settings.enableClipboardMonitoring, navigateTo]);

  return { detectedUrl };
}
