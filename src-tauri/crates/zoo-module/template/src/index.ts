import { defineModule } from 'zoo-module';

export default defineModule({
  async resolve(url, zoo) {
    const response = await fetch(url);
    if (response.status === 404) {
      throw zoo.fail('file not found', { hint: 'the link may have expired or been deleted' });
    }
    const doc = new DOMParser().parseFromString(await response.text(), 'text/html');

    const href = doc.querySelector('a.download')?.getAttribute('href');
    if (!href) {
      throw zoo.fail('download link not found', { hint: 'the page layout may have changed' });
    }
    return {
      url: new URL(href, response.url).href,
      filename: doc.querySelector('h1.filename')?.textContent?.trim() || undefined,
      headers: { Referer: response.url },
    };
  },
});
