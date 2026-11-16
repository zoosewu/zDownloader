// Add Download Page - Streamlined add download UI
import React from 'react';
import { Button } from '@mui/material';
import { useTranslation } from 'react-i18next';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import { useNavigation } from '../hooks/useNavigation';
import AddDownloadForm from '../components/AddDownloadForm';
import PageLayout from '../components/PageLayout';

const AddDownloadPage: React.FC = () => {
  const { t } = useTranslation();
  const { goBack, navigationParams } = useNavigation();
  const initialUrl = navigationParams?.url || '';

  return (
    <PageLayout
      header={
        <Button onClick={goBack} startIcon={<ArrowBackIcon />} color="inherit">
          {t('common.back')}
        </Button>
      }
      contentSx={{ p: 2 }}
    >
      <AddDownloadForm initialUrl={initialUrl} onCancel={goBack} />
    </PageLayout>
  );
};

export default AddDownloadPage;
