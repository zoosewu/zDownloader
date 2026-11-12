// Settings Page - App settings and Proxy management
import { Box, Tabs, Tab } from '@mui/material';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import ProxySettings from '../components/ProxySettings';
import GeneralSettings from '../components/GeneralSettings';
import AdvancedSettings from '../components/AdvancedSettings';
import ModulesSettings from '../components/modules/ModulesSettings';
import PageLayout from '../components/PageLayout';

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
      {value === index && <Box sx={{ p: 3 }}>{children}</Box>}
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
    <PageLayout
      header={
        <Tabs value={currentTab} onChange={handleTabChange} sx={{ width: '100%' }}>
          <Tab label={t('settings.general')} />
          <Tab label={t('settings.proxy')} />
          <Tab label={t('modules.title')} />
          <Tab label={t('settings.advanced')} />
        </Tabs>
      }
    >
      <TabPanel value={currentTab} index={0}>
        <GeneralSettings />
      </TabPanel>

      <TabPanel value={currentTab} index={1}>
        <ProxySettings />
      </TabPanel>

      <TabPanel value={currentTab} index={2}>
        <ModulesSettings />
      </TabPanel>

      <TabPanel value={currentTab} index={3}>
        <AdvancedSettings />
      </TabPanel>
    </PageLayout>
  );
}
