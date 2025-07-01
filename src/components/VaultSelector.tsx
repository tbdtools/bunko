import { useState, useEffect } from 'react';
import { 
  Modal, 
  Stack, 
  Title, 
  Text, 
  Button, 
  Group, 
  Card, 
  TextInput, 
  ActionIcon,
  ScrollArea,
  Box,
  UnstyledButton 
} from '@mantine/core';
import { 
  IconPlus, 
  IconFolderOpen, 
  IconFolder, 
  IconTrash 
} from '@tabler/icons-react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

interface Vault {
  name: string;
  path: string;
  lastOpened?: string;
  noteCount?: number;
}

interface VaultSelectorProps {
  opened: boolean;
  onVaultSelected: (vaultPath: string) => void;
}

export default function VaultSelector({ opened, onVaultSelected }: VaultSelectorProps) {
  const [vaults, setVaults] = useState<Vault[]>([]);
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [newVaultName, setNewVaultName] = useState('');
  const [newVaultPath, setNewVaultPath] = useState('');
  const [isCreating, setIsCreating] = useState(false);

  useEffect(() => {
    if (opened) {
      loadVaults();
    }
  }, [opened]);

  const loadVaults = async () => {
    try {
      const vaultList = await invoke<Vault[]>('list_vaults');
      setVaults(vaultList);
    } catch (error) {
      console.error('Failed to load vaults:', error);
      // If no vaults exist, start with empty list
      setVaults([]);
    }
  };

  const createNewVault = async () => {
    if (!newVaultName.trim() || !newVaultPath.trim()) return;

    setIsCreating(true);
    try {
      await invoke('create_vault', {
        name: newVaultName.trim(),
        path: newVaultPath.trim()
      });
      
      // Refresh vault list and select the new vault
      await loadVaults();
      onVaultSelected(newVaultPath.trim());
      
      setShowCreateForm(false);
      setNewVaultName('');
      setNewVaultPath('');
    } catch (error) {
      console.error('Failed to create vault:', error);
    } finally {
      setIsCreating(false);
    }
  };

  const selectVaultLocation = async () => {
    try {
      const selected = await open({
        directory: true,
        title: 'Select Vault Location'
      });
      
      if (selected) {
        setNewVaultPath(selected);
      }
    } catch (error) {
      console.error('Failed to select directory:', error);
    }
  };

  const openExistingVault = async () => {
    try {
      const selected = await open({
        directory: true,
        title: 'Select Vault Folder'
      });
      
      if (selected) {
        // Check if it's a valid vault (has .bunko directory)
        try {
          await invoke('validate_vault', { path: selected });
          onVaultSelected(selected);
        } catch (error) {
          console.error('Invalid vault selected:', error);
          // Could show error message to user
        }
      }
    } catch (error) {
      console.error('Failed to open vault:', error);
    }
  };

  const deleteVault = async (vaultPath: string) => {
    try {
      await invoke('remove_vault', { path: vaultPath });
      await loadVaults();
    } catch (error) {
      console.error('Failed to remove vault:', error);
    }
  };

  return (
    <Modal
      opened={opened}
      onClose={() => {}} // Prevent closing without selecting vault
      title="Select Vault"
      size="md"
      centered
      closeOnClickOutside={false}
      closeOnEscape={false}
      withCloseButton={false}
    >
      <Stack gap="md">
        <Text size="sm" c="dimmed">
          Choose a vault to work with, or create a new one.
        </Text>

        {!showCreateForm ? (
          <>
            {/* Existing Vaults */}
            {vaults.length > 0 && (
              <>
                <Title order={6}>Recent Vaults</Title>
                <ScrollArea style={{ maxHeight: 300 }}>
                  <Stack gap="xs">
                    {vaults.map((vault) => (
                      <Card key={vault.path} padding="md" withBorder>
                        <Group justify="space-between">
                          <UnstyledButton
                            onClick={() => onVaultSelected(vault.path)}
                            style={{ flex: 1 }}
                          >
                            <Box>
                              <Group gap="sm">
                                <IconFolder size={20} />
                                <div>
                                  <Text fw={500}>{vault.name}</Text>
                                  <Text size="xs" c="dimmed">{vault.path}</Text>
                                  {vault.noteCount !== undefined && (
                                    <Text size="xs" c="dimmed">
                                      {vault.noteCount} notes
                                    </Text>
                                  )}
                                </div>
                              </Group>
                            </Box>
                          </UnstyledButton>
                          <ActionIcon
                            variant="subtle"
                            color="red"
                            onClick={() => deleteVault(vault.path)}
                          >
                            <IconTrash size={16} />
                          </ActionIcon>
                        </Group>
                      </Card>
                    ))}
                  </Stack>
                </ScrollArea>
              </>
            )}

            {/* Action Buttons */}
            <Group justify="center" mt="md">
              <Button
                leftSection={<IconPlus size={16} />}
                onClick={() => setShowCreateForm(true)}
                variant="filled"
              >
                Create New Vault
              </Button>
              <Button
                leftSection={<IconFolderOpen size={16} />}
                onClick={openExistingVault}
                variant="outline"
              >
                Open Existing Vault
              </Button>
            </Group>
          </>
        ) : (
          <>
            {/* Create New Vault Form */}
            <Title order={6}>Create New Vault</Title>
            <Stack gap="md">
              <TextInput
                label="Vault Name"
                placeholder="My Notes"
                value={newVaultName}
                onChange={(e) => setNewVaultName(e.currentTarget.value)}
                required
              />
              
              <Group>
                <TextInput
                  label="Location"
                  placeholder="Select folder..."
                  value={newVaultPath}
                  onChange={(e) => setNewVaultPath(e.currentTarget.value)}
                  style={{ flex: 1 }}
                  required
                />
                <Button
                  onClick={selectVaultLocation}
                  variant="outline"
                  mt="auto"
                >
                  Browse
                </Button>
              </Group>

              <Group justify="apart" mt="md">
                <Button
                  variant="subtle"
                  onClick={() => {
                    setShowCreateForm(false);
                    setNewVaultName('');
                    setNewVaultPath('');
                  }}
                >
                  Cancel
                </Button>
                <Button
                  onClick={createNewVault}
                  loading={isCreating}
                  disabled={!newVaultName.trim() || !newVaultPath.trim()}
                >
                  Create Vault
                </Button>
              </Group>
            </Stack>
          </>
        )}
      </Stack>
    </Modal>
  );
}