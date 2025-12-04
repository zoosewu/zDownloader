// Download detail panel component - displays detailed information for selected download
import React, { useState } from 'react';
import { Box, Paper, Tabs, Tab, Typography } from '@mui/material';
import { useTranslation } from 'react-i18next';
import { DownloadTask } from '../types/download';
import BasicInfoTab from './BasicInfoTab';
import ErrorInfoTab from './ErrorInfoTab';
import AdvancedInfoTab from './AdvancedInfoTab';
import SegmentVisualization from './SegmentVisualization';

interface DownloadDetailPanelProps {
  task: DownloadTask | null;
}

/**
 * Download detail panel with tabs for different types of information
 * Displays below the download table when a task is selected
 */
const DownloadDetailPanel: React.FC<DownloadDetailPanelProps> = ({ task }) => {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState(0);

  // Reset to first tab when task changes
  React.useEffect(() => {
    setActiveTab(0);
  }, [task?.id]);

  // Handle tab change
  const handleTabChange = (_event: React.SyntheticEvent, newValue: number) => {
    setActiveTab(newValue);
  };

  // No task selected - show empty state
  if (!task) {
    return (
      <Paper
        elevation={1}
        sx={{
          minHeight: 250,
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
        }}
      >
        <Typography variant="body1" color="text.secondary">
          {t('downloads.details.selectItem')}
        </Typography>
      </Paper>
    );
  }

  return (
    <Paper
      elevation={0}
      sx={{
        width: '100%',
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      {/* Tabs Header - Fixed at top */}
      <Box sx={{ borderBottom: 1, borderColor: 'divider', flexShrink: 0 }}>
        <Tabs
          value={activeTab}
          onChange={handleTabChange}
          aria-label="download detail tabs"
          sx={{ paddingX: 2 }}
        >
          <Tab label={t('downloads.details.basicInfo')} />
          <Tab label={t('downloads.details.segmentVisualization')} />
          <Tab label={t('downloads.details.errorInfo')} />
          <Tab label={t('downloads.details.advancedInfo')} />
        </Tabs>
      </Box>

      {/* Tab Content - Scrollable */}
      <Box
        sx={{
          flex: 1,
          overflow: 'auto',
        }}
      >
        <Box role="tabpanel" hidden={activeTab !== 0}>
          {activeTab === 0 && <BasicInfoTab task={task} />}
        </Box>

        <Box role="tabpanel" hidden={activeTab !== 1}>
          {activeTab === 1 && <SegmentVisualization task={task} />}
        </Box>

        <Box role="tabpanel" hidden={activeTab !== 2}>
          {activeTab === 2 && <ErrorInfoTab task={task} />}
        </Box>

        <Box role="tabpanel" hidden={activeTab !== 3}>
          {activeTab === 3 && <AdvancedInfoTab task={task} />}
        </Box>
      </Box>
    </Paper>
  );
};

export default DownloadDetailPanel;
