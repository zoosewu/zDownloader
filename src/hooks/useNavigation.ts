// useNavigation hook - manages page navigation
import { useAtom } from 'jotai';
import { useCallback } from 'react';
import {
  currentPageAtom,
  previousPageAtom,
  navigationParamsAtom,
  PageType,
} from '../stores/navigationStore';

export interface UseNavigationReturn {
  currentPage: PageType;
  previousPage: PageType | null;
  navigationParams: Record<string, any>;
  navigateTo: (page: PageType, params?: Record<string, any>) => void;
  goBack: () => void;
  canGoBack: boolean;
}

export function useNavigation(): UseNavigationReturn {
  const [currentPage, setCurrentPage] = useAtom(currentPageAtom);
  const [previousPage, setPreviousPage] = useAtom(previousPageAtom);
  const [params, setParams] = useAtom(navigationParamsAtom);

  const navigateTo = useCallback(
    (page: PageType, newParams?: Record<string, any>) => {
      // Store current page as previous
      setPreviousPage(currentPage);
      // Navigate to new page
      setCurrentPage(page);
      // Set navigation parameters
      setParams(newParams || {});
    },
    [currentPage, setCurrentPage, setPreviousPage, setParams]
  );

  const goBack = useCallback(() => {
    if (previousPage) {
      setCurrentPage(previousPage);
      setPreviousPage(null);
      setParams({});
    }
  }, [previousPage, setCurrentPage, setPreviousPage, setParams]);

  return {
    currentPage,
    previousPage,
    navigationParams: params,
    navigateTo,
    goBack,
    canGoBack: previousPage !== null,
  };
}
