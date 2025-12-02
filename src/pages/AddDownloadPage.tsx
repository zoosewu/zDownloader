// Add Download Page - Streamlined add download UI
import React from 'react';
import { Container, Box, Typography, Button } from '@mui/material';
import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import { useNavigation } from '../hooks/useNavigation';
import AddDownloadForm from '../components/AddDownloadForm';

const AddDownloadPage: React.FC = () => {
  const { goBack, navigationParams } = useNavigation();
  const initialUrl = navigationParams?.url || '';

  return (
    <Container maxWidth="lg" sx={{ paddingY: 4 }}>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 3 }}>
        <Typography variant="h4">新增下載</Typography>
        <Button onClick={goBack} startIcon={<ArrowBackIcon />}>
          返回
        </Button>
      </Box>

      <AddDownloadForm initialUrl={initialUrl} onCancel={goBack} />
    </Container>
  );
};

export default AddDownloadPage;
