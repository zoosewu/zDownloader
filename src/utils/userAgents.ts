// User-Agents people pick most: browsers some sites insist on, and Baidu's own client.
export const USER_AGENT_PRESETS: Array<{ label: string; value: string }> = [
  {
    label: 'Chrome (Windows)',
    value: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36',
  },
  {
    label: 'Edge (Windows)',
    value: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36 Edg/142.0.0.0',
  },
  {
    label: 'Firefox (Windows)',
    value: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:145.0) Gecko/20100101 Firefox/145.0',
  },
  {
    label: 'Safari (macOS)',
    value: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.6 Safari/605.1.15',
  },
  { label: '百度網盤 (netdisk)', value: 'netdisk;7.42.0.5;PC;PC-Windows;10.0.22631;WindowsBaiduYunGuanJia' },
];
