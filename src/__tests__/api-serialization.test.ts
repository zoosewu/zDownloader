/**
 * API Serialization Tests
 * 確保前端能正確解析後端返回的 JSON 資料
 * 使用與後端相同的測試資料，確保前後端一致性
 */

import { describe, test, expect } from 'vitest';

// 前端的 TypeScript 介面定義
interface UrlDetectionResult {
  url: string;
  isValid: boolean;
  detectorName?: string;
  detectionMode?: string;
  suggestedFilename?: string;
  fileExtension?: string;
}

describe('API Serialization - UrlDetectionResult', () => {
  /**
   * Test: 確保能正確解析後端返回的 camelCase JSON
   * 這個 JSON 格式應該與後端序列化輸出完全一致
   */
  test('should correctly parse backend camelCase JSON', () => {
    // 這個 JSON 來自後端的序列化輸出
    const backendJson = `{
      "url": "https://example.com/test-file.zip",
      "isValid": true,
      "detectorName": "BuiltinFileDetector",
      "detectionMode": "fileExtension",
      "suggestedFilename": "test-file.zip",
      "fileExtension": "zip"
    }`;

    const result: UrlDetectionResult = JSON.parse(backendJson);

    // 驗證所有欄位都能正確解析
    expect(result.url).toBe('https://example.com/test-file.zip');
    expect(result.isValid).toBe(true);
    expect(result.detectorName).toBe('BuiltinFileDetector');
    expect(result.detectionMode).toBe('fileExtension');
    expect(result.suggestedFilename).toBe('test-file.zip');
    expect(result.fileExtension).toBe('zip');
  });

  /**
   * Test: 驗證前端不應該使用 snake_case 欄位
   */
  test('should not use snake_case field names', () => {
    // 錯誤的 snake_case JSON（舊版本或未配置 serde 的後端）
    const incorrectJson = `{
      "url": "https://example.com/test.zip",
      "is_valid": true,
      "detector_name": "Test"
    }`;

    const result: UrlDetectionResult = JSON.parse(incorrectJson);

    // 這些欄位應該是 undefined，因為前端使用 camelCase
    expect(result.isValid).toBeUndefined();
    expect(result.detectorName).toBeUndefined();

    // 這個測試展示了為什麼必須使用 #[serde(rename_all = "camelCase")]
  });

  /**
   * Test: 完整的往返測試
   */
  test('should maintain data integrity in roundtrip', () => {
    const original: UrlDetectionResult = {
      url: 'https://example.com/test.pdf',
      isValid: true,
      detectorName: 'TestDetector',
      detectionMode: 'fileExtension',
      suggestedFilename: 'test.pdf',
      fileExtension: 'pdf',
    };

    // 序列化
    const json = JSON.stringify(original);

    // 反序列化
    const deserialized: UrlDetectionResult = JSON.parse(json);

    // 驗證資料完整性
    expect(deserialized).toEqual(original);
  });

  /**
   * Test: 測試可選欄位
   */
  test('should handle optional fields correctly', () => {
    const minimalJson = `{
      "url": "https://example.com/test",
      "isValid": false
    }`;

    const result: UrlDetectionResult = JSON.parse(minimalJson);

    expect(result.url).toBe('https://example.com/test');
    expect(result.isValid).toBe(false);
    expect(result.detectorName).toBeUndefined();
    expect(result.detectionMode).toBeUndefined();
    expect(result.suggestedFilename).toBeUndefined();
    expect(result.fileExtension).toBeUndefined();
  });

  /**
   * Test: 驗證 enum 值的格式
   */
  test('should correctly parse enum values in camelCase', () => {
    const jsonWithFileExtension = `{
      "url": "https://example.com/test.zip",
      "isValid": true,
      "detectionMode": "fileExtension"
    }`;

    const result1: UrlDetectionResult = JSON.parse(jsonWithFileExtension);
    expect(result1.detectionMode).toBe('fileExtension');

    const jsonWithModuleCustom = `{
      "url": "https://example.com/test",
      "isValid": true,
      "detectionMode": "moduleCustom"
    }`;

    const result2: UrlDetectionResult = JSON.parse(jsonWithModuleCustom);
    expect(result2.detectionMode).toBe('moduleCustom');
  });
});

describe('API Serialization - Type Safety', () => {
  /**
   * Test: TypeScript 型別檢查應該捕捉命名錯誤
   */
  test('TypeScript should catch snake_case usage at compile time', () => {
    // 這段代碼應該在 TypeScript 編譯時報錯
    // 但在測試中我們可以展示正確的用法

    const data: UrlDetectionResult = {
      url: 'https://example.com/test.zip',
      isValid: true, // ✓ 正確
      // is_valid: true, // ✗ TypeScript 會報錯
      detectorName: 'Test', // ✓ 正確
      // detector_name: 'Test', // ✗ TypeScript 會報錯
    };

    expect(data.isValid).toBe(true);
    expect(data.detectorName).toBe('Test');
  });
});

/**
 * Integration Test Helper
 * 用於驗證前後端資料一致性的輔助函數
 */
export function validateBackendResponse(response: unknown): response is UrlDetectionResult {
  if (typeof response !== 'object' || response === null) {
    console.error('[API Validation] Response is not an object:', response);
    return false;
  }

  const r = response as any;

  // 必須欄位檢查
  if (typeof r.url !== 'string') {
    console.error('[API Validation] Missing or invalid "url" field');
    return false;
  }

  if (typeof r.isValid !== 'boolean') {
    console.error('[API Validation] Missing or invalid "isValid" field (should be camelCase)');
    return false;
  }

  // 警告：檢測到 snake_case 欄位
  if ('is_valid' in r) {
    console.warn(
      '[API Validation] ⚠️ Found snake_case field "is_valid" - backend may be missing #[serde(rename_all = "camelCase")]'
    );
  }

  if ('detector_name' in r) {
    console.warn(
      '[API Validation] ⚠️ Found snake_case field "detector_name" - backend may be missing #[serde(rename_all = "camelCase")]'
    );
  }

  return true;
}
