// Add Download Page - Streamlined add download UI
import React from 'react';
import { Box, Button } from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import { useNavigation } from '../hooks/useNavigation';
import AddDownloadForm from '../components/AddDownloadForm';
import PageLayout from '../components/PageLayout';

const AddDownloadPage: React.FC = () => {
  const { goBack, navigationParams } = useNavigation();
  const initialUrl = navigationParams?.url || '';

  return (
    <PageLayout
      header={
        <Box sx={{ display: 'flex', justifyContent: 'flex-end', width: '100%' }}>
          <Button onClick={goBack} startIcon={<ArrowBackIcon />} size="small">
            返回
          </Button>
        </Box>
      }
      contentSx={{ p: 3 }}
    >
      <AddDownloadForm initialUrl={initialUrl} onCancel={goBack} />
    </PageLayout>
  );
};

export default AddDownloadPage;
