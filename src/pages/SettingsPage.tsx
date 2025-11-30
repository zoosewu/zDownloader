// Settings Page - App settings and Proxy management
import { Box, Container, Typography, Tabs, Tab } from '@mui/material';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import ProxySettings from '../components/ProxySettings';
import GeneralSettings from '../components/GeneralSettings';
import AdvancedSettings from '../components/AdvancedSettings';

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
  const { t } = useTranslation();
  const [currentTab, setCurrentTab] = useState(0);

  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setCurrentTab(newValue);
  };

  return (
    <Container maxWidth="xl" sx={{ py: 4 }}>
      <Typography variant="h4" component="h1" gutterBottom>
        {t('settings.title')}
      </Typography>

      <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 2 }}>
        <Tabs value={currentTab} onChange={handleTabChange}>
          <Tab label={t('settings.general')} />
          <Tab label={t('settings.proxy')} />
          <Tab label={t('settings.advanced')} />
        </Tabs>
      </Box>

      <TabPanel value={currentTab} index={0}>
        <GeneralSettings />
      </TabPanel>

      <TabPanel value={currentTab} index={1}>
        <ProxySettings />
      </TabPanel>

      <TabPanel value={currentTab} index={2}>
        <AdvancedSettings />
      </TabPanel>
    </Container>
  );
}
