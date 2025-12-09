/// <reference types="../@types/zoo-downloader" />

/**
 * Simple Example Module - 最小化的教學範例
 *
 * 這個模組展示了最基本的模組結構和功能。
 *
 * @implements {import('zoo-downloader').SiteModule}
 */
export class SimpleExampleModule {
  /**
   * 檢查此模組是否能處理給定的 URL
   *
   * @param {string} url - 要檢查的 URL
   * @returns {boolean} true 表示此模組可以處理此 URL
   *
   * @example
   * canHandle('https://example.com/file/abc123') // returns true
   * canHandle('https://other-site.com/file') // returns false
   */
  canHandle(url) {
    // 簡單的字串匹配檢查
    return url.includes('example.com/file/') ||
           url.includes('example.com/download/');
  }

  /**
   * 從給定的 URL 提取下載資訊
   *
   * @param {string} url - 檔案頁面的 URL
   * @param {import('zoo-downloader').ModuleContext} context - 模組上下文，提供 HTTP 客戶端、日誌等功能
   * @returns {Promise<import('zoo-downloader').DownloadInfo>} 下載資訊物件
   *
   * @example
   * const info = await extractDownloadInfo(
   *   'https://example.com/file/abc123',
   *   context
   * );
   * // Returns:
   * // {
   * //   downloadUrl: 'https://cdn.example.com/files/abc123.zip',
   * //   filename: 'example-file.zip',
   * //   filesize: 1048576,
   * //   headers: { 'User-Agent': '...' }
   * // }
   */
  async extractDownloadInfo(url, context) {
    // 步驟 1: 記錄開始處理
    context.log.info(`開始處理 URL: ${url}`);

    try {
      // 步驟 2: 獲取檔案頁面的 HTML
      context.log.debug('正在獲取檔案頁面...');
      const response = await context.http.get(url);

      // 檢查 HTTP 狀態
      if (!response.ok) {
        throw new Error(`HTTP 錯誤: ${response.status}`);
      }

      const html = response.body;

      // 步驟 3: 從 HTML 中提取資訊
      // 注意：這些正則表達式是示範用，實際網站的 HTML 結構會不同

      // 提取檔案名稱
      const filenameMatch = html.match(/<h1[^>]*>(.+?)<\/h1>/);
      const filename = filenameMatch ? filenameMatch[1] : 'download.bin';

      // 提取下載連結
      const downloadUrlMatch = html.match(/href="(https:\/\/cdn\.example\.com\/[^"]+)"/);
      if (!downloadUrlMatch) {
        throw new Error('找不到下載連結');
      }
      const downloadUrl = downloadUrlMatch[1];

      // 提取檔案大小（可選）
      const sizeMatch = html.match(/(\d+)\s*MB/);
      const filesize = sizeMatch ? parseInt(sizeMatch[1]) * 1024 * 1024 : null;

      // 步驟 4: 返回下載資訊
      context.log.info(`成功提取: ${filename}`);

      return {
        downloadUrl: downloadUrl,
        filename: filename,
        filesize: filesize,
        headers: {
          // 設置 User-Agent 避免被網站阻擋
          'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
          // 設置 Referer（某些網站會檢查）
          'Referer': url
        }
      };

    } catch (error) {
      // 記錄錯誤並重新拋出
      context.log.error(`提取失敗: ${error.message}`);
      throw error;
    }
  }
}

// 必須使用預設導出
export default SimpleExampleModule;
