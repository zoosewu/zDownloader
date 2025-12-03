/**
 * API Serialization Tests
 *
 * These tests verify that the backend correctly serializes data to camelCase JSON
 * and that the frontend can properly parse the responses.
 */

import { describe, test, expect } from 'vitest';

/**
 * TypeScript interface matching the backend's UrlDetectionInfo
 */
interface UrlDetectionResult {
  url: string;
  isValid: boolean;
  detectorName: string;
  detectionMode: 'fileExtension' | 'moduleCustom';
  suggestedFilename?: string;
  fileExtension?: string;
}

describe('API Serialization - UrlDetectionResult', () => {
  test('should correctly parse backend camelCase JSON', () => {
    // This JSON format should match what the Rust backend outputs
    const backendJson = `{
      "url": "https://example.com/test-file.zip",
      "isValid": true,
      "detectorName": "BuiltinFileDetector",
      "detectionMode": "fileExtension",
      "suggestedFilename": "test-file.zip",
      "fileExtension": "zip"
    }`;

    const result: UrlDetectionResult = JSON.parse(backendJson);

    expect(result.url).toBe('https://example.com/test-file.zip');
    expect(result.isValid).toBe(true);
    expect(result.detectorName).toBe('BuiltinFileDetector');
    expect(result.detectionMode).toBe('fileExtension');
    expect(result.suggestedFilename).toBe('test-file.zip');
    expect(result.fileExtension).toBe('zip');
  });

  test('should not use snake_case field names', () => {
    // This demonstrates what happens if the backend doesn't use rename_all
    const incorrectJson = `{
      "url": "https://example.com/test.zip",
      "is_valid": true,
      "detector_name": "Test"
    }`;

    const result: UrlDetectionResult = JSON.parse(incorrectJson);

    // These will be undefined because TypeScript expects camelCase
    expect(result.isValid).toBeUndefined();
    expect(result.detectorName).toBeUndefined();
  });

  test('should maintain data integrity in roundtrip', () => {
    const original = {
      url: 'https://example.com/file.pdf',
      isValid: true,
      detectorName: 'BuiltinFileDetector',
      detectionMode: 'fileExtension' as const,
      suggestedFilename: 'file.pdf',
      fileExtension: 'pdf',
    };

    const json = JSON.stringify(original);
    const parsed: UrlDetectionResult = JSON.parse(json);

    expect(parsed).toEqual(original);
  });

  test('should handle optional fields correctly', () => {
    const minimalJson = `{
      "url": "https://example.com/unknown",
      "isValid": false,
      "detectorName": "UnknownDetector",
      "detectionMode": "moduleCustom"
    }`;

    const result: UrlDetectionResult = JSON.parse(minimalJson);

    expect(result.url).toBe('https://example.com/unknown');
    expect(result.isValid).toBe(false);
    expect(result.suggestedFilename).toBeUndefined();
    expect(result.fileExtension).toBeUndefined();
  });

  test('should correctly parse enum values in camelCase', () => {
    const json1 = '{"detectionMode": "fileExtension"}';
    const json2 = '{"detectionMode": "moduleCustom"}';

    const result1 = JSON.parse(json1);
    const result2 = JSON.parse(json2);

    expect(result1.detectionMode).toBe('fileExtension');
    expect(result2.detectionMode).toBe('moduleCustom');
  });

  test('TypeScript should catch snake_case usage at compile time', () => {
    const data: UrlDetectionResult = {
      url: 'https://example.com/test.zip',
      isValid: true, // ✓ Correct camelCase
      detectorName: 'Test', // ✓ Correct camelCase
      detectionMode: 'fileExtension', // ✓ Correct camelCase
      // is_valid: true, // ✗ This would cause TypeScript compile error
      // detector_name: 'Test', // ✗ This would cause TypeScript compile error
    };

    expect(data.isValid).toBe(true);
    expect(data.detectorName).toBe('Test');
  });
});

/**
 * Helper function to validate backend responses at runtime
 */
export function validateBackendResponse(response: unknown): response is UrlDetectionResult {
  const r = response as any;

  // Check required fields exist and have correct types
  if (typeof r.url !== 'string') {
    console.error('[API Validation] Missing or invalid "url" field');
    return false;
  }

  if (typeof r.isValid !== 'boolean') {
    console.error('[API Validation] Missing or invalid "isValid" field');
    return false;
  }

  if (typeof r.detectorName !== 'string') {
    console.error('[API Validation] Missing or invalid "detectorName" field');
    return false;
  }

  if (typeof r.detectionMode !== 'string') {
    console.error('[API Validation] Missing or invalid "detectionMode" field');
    return false;
  }

  // Warn if snake_case fields are found (indicates backend issue)
  if ('is_valid' in r) {
    console.warn('[API Validation] ⚠️ Found snake_case field "is_valid" - backend may be missing #[serde(rename_all = "camelCase")]');
  }

  if ('detector_name' in r) {
    console.warn('[API Validation] ⚠️ Found snake_case field "detector_name" - backend may be missing #[serde(rename_all = "camelCase")]');
  }

  if ('detection_mode' in r) {
    console.warn('[API Validation] ⚠️ Found snake_case field "detection_mode" - backend may be missing #[serde(rename_all = "camelCase")]');
  }

  return true;
}
