// Collapsible field component - displays long content with expand/collapse functionality
import React, { useState } from 'react';
import { Box, Typography, IconButton } from '@mui/material';
import { ChevronRight as ChevronRightIcon, ExpandMore as ExpandMoreIcon } from '@mui/icons-material';

interface CollapsibleFieldProps {
  label: string;
  content: string;
  defaultExpanded?: boolean;
  contentStyle?: 'text' | 'monospace' | 'link';
}

/**
 * Collapsible field component for displaying long content
 * - Collapsed: Single line with arrow, label, and truncated content
 * - Expanded: Two lines with arrow, label on first line, full content on second line
 */
const CollapsibleField: React.FC<CollapsibleFieldProps> = ({
  label,
  content,
  defaultExpanded = false,
  contentStyle = 'text',
}) => {
  const [expanded, setExpanded] = useState(defaultExpanded);

  const handleToggle = () => {
    setExpanded(!expanded);
  };

  // Content styling based on type
  const getContentSx = () => {
    const baseSx = {
      wordBreak: 'break-all' as const,
      overflowWrap: 'break-word' as const,
    };

    if (contentStyle === 'monospace') {
      return {
        ...baseSx,
        fontFamily: 'monospace',
        backgroundColor: 'action.hover',
        padding: 1,
        borderRadius: 1,
        fontSize: '0.875rem',
      };
    }

    if (contentStyle === 'link') {
      return {
        ...baseSx,
        color: 'primary.main',
        fontSize: '0.875rem',
      };
    }

    return {
      ...baseSx,
      fontSize: '0.875rem',
    };
  };

  if (expanded) {
    // Expanded state: Two lines
    return (
      <Box sx={{ marginBottom: 2 }}>
        {/* First line: Arrow + Label */}
        <Box
          sx={{
            display: 'flex',
            alignItems: 'center',
            cursor: 'pointer',
            '&:hover': {
              backgroundColor: 'action.hover',
            },
            borderRadius: 1,
            padding: '4px 0',
          }}
          onClick={handleToggle}
        >
          <IconButton size="small" sx={{ padding: 0, marginRight: 0.5 }}>
            <ExpandMoreIcon fontSize="small" />
          </IconButton>
          <Typography variant="subtitle2" color="text.secondary">
            {label}
          </Typography>
        </Box>

        {/* Second line: Full content */}
        <Box sx={{ marginTop: 1, marginLeft: 3 }}>
          <Typography variant="body2" sx={getContentSx()}>
            {content}
          </Typography>
        </Box>
      </Box>
    );
  }

  // Collapsed state: Single line with truncated content
  return (
    <Box
      sx={{
        display: 'flex',
        alignItems: 'center',
        cursor: 'pointer',
        marginBottom: 2,
        '&:hover': {
          backgroundColor: 'action.hover',
        },
        borderRadius: 1,
        padding: '4px 0',
      }}
      onClick={handleToggle}
    >
      {/* Arrow */}
      <IconButton size="small" sx={{ padding: 0, marginRight: 0.5 }}>
        <ChevronRightIcon fontSize="small" />
      </IconButton>

      {/* Label */}
      <Typography
        variant="subtitle2"
        color="text.secondary"
        sx={{ marginRight: 2, minWidth: 80, flexShrink: 0 }}
      >
        {label}
      </Typography>

      {/* Truncated content */}
      <Typography
        variant="body2"
        sx={{
          flex: 1,
          overflow: 'hidden',
          textOverflow: 'ellipsis',
          whiteSpace: 'nowrap',
          fontSize: '0.875rem',
          color: contentStyle === 'link' ? 'primary.main' : 'text.primary',
        }}
      >
        {content}
      </Typography>
    </Box>
  );
};

export default CollapsibleField;
