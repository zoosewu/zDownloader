// Page layout component - provides consistent full-width layout for all pages
import React from 'react';
import { Box } from '@mui/material';

interface PageLayoutProps {
  /**
   * Optional header content (tabs, buttons, etc.)
   * Will be fixed at the top with border-bottom
   */
  header?: React.ReactNode;

  /**
   * Main page content
   * Will be scrollable if it exceeds the available height
   */
  children: React.ReactNode;

  /**
   * Additional sx props for the content area
   */
  contentSx?: any;

  /**
   * Additional sx props for the header area
   */
  headerSx?: any;
}

/**
 * Standard page layout component for consistent full-width design
 * - Fills 100% width and height
 * - Optional fixed header
 * - Scrollable content area
 * - Matches the download list layout structure
 */
const PageLayout: React.FC<PageLayoutProps> = ({
  header,
  children,
  contentSx = {},
  headerSx = {},
}) => {
  return (
    <Box
      sx={{
        width: '100%',
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        position: 'relative',
      }}
    >
      {/* Optional Header - fixed at top */}
      {header && (
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            px: 2,
            py: 1.5,
            borderBottom: 1,
            borderColor: 'divider',
            backgroundColor: 'background.paper',
            flexShrink: 0,
            ...headerSx,
          }}
        >
          {header}
        </Box>
      )}

      {/* Scrollable Content Area */}
      <Box
        sx={{
          flex: 1,
          overflow: 'auto',
          ...contentSx,
        }}
      >
        {children}
      </Box>
    </Box>
  );
};

export default PageLayout;
