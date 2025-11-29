// Settings Page - App settings and Proxy management
import { Box, Container, Typography, Tabs, Tab } from '@mui/material';
import { useState } from 'react';
import ProxySettings from '../components/ProxySettings';

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;

  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`settings-tabpanel-${index}`}
      aria-labelledby={`settings-tab-${index}`}
      {...other}
    >
      {value === index && <Box sx={{ py: 3 }}>{children}</Box>}
    </div>
  );
}

export default function SettingsPage() {
  const [currentTab, setCurrentTab] = useState(0);

  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setCurrentTab(newValue);
  };

  return (
    <Container maxWidth="xl" sx={{ py: 4 }}>
      <Typography variant="h4" component="h1" gutterBottom>
        設定
      </Typography>

      <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 2 }}>
        <Tabs value={currentTab} onChange={handleTabChange}>
          <Tab label="Proxy 管理" />
          <Tab label="一般設定" />
          <Tab label="進階設定" />
        </Tabs>
      </Box>

      <TabPanel value={currentTab} index={0}>
        <ProxySettings />
      </TabPanel>

      <TabPanel value={currentTab} index={1}>
        <Box sx={{ p: 3, textAlign: 'center', color: 'text.secondary' }}>
          <Typography>一般設定功能開發中...</Typography>
        </Box>
      </TabPanel>

      <TabPanel value={currentTab} index={2}>
        <Box sx={{ p: 3, textAlign: 'center', color: 'text.secondary' }}>
          <Typography>進階設定功能開發中...</Typography>
        </Box>
      </TabPanel>
    </Container>
  );
}
