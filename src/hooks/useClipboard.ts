// useClipboard hook - Monitor clipboard for download URLs and auto-navigate
import { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
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

export function useClipboard() {
  const [detectedUrl, setDetectedUrl] = useState<string | null>(null);
  const lastContentRef = useRef<string>('');
  const processingRef = useRef<boolean>(false);
  const { navigateTo } = useNavigation();
  const settings = useAtomValue(settingsAtom);

  useEffect(() => {
    if (!settings.enableClipboardMonitoring) return;

    const interval = setInterval(async () => {
      // Skip if still processing previous content
      if (processingRef.current) return;

      try {
        const content = await invoke<string>('get_clipboard_content');

        if (content !== lastContentRef.current) {
          processingRef.current = true;
          lastContentRef.current = content;

          // Quick client-side validation using urlDetector
          if (!urlDetector.isValidUrl(content)) {
            processingRef.current = false;
            return;
          }

          // Pre-filter: only check URLs with common file extensions
          // This reduces unnecessary backend calls
          if (!urlDetector.hasCommonFileExtension(content)) {
            processingRef.current = false;
            return;
          }

          try {
            const result = await invoke<UrlDetectionResult>('detect_url', { url: content });

            if (result.isValid) {
              setDetectedUrl(content);
              navigateTo('add-download', { url: content });
            }
          } catch (e) {
            // Silent fail - URL not detected as downloadable
          } finally {
            processingRef.current = false;
          }
        }
      } catch (e) {
        // Silent fail or log error
        processingRef.current = false;
      }
    }, 400); // 400ms polling interval

    return () => clearInterval(interval);
  }, [settings.enableClipboardMonitoring, navigateTo]);

  return { detectedUrl };
}
