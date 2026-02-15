import { useState, useEffect, useCallback } from "react";
import { Plus, Users } from "lucide-react";
import { commands } from "@/lib/tauri";
import { EmptyState } from "@/components/EmptyState/EmptyState";
import type { ChatRoom as ChatRoomType } from "@/lib/tauri";
import ChatRoom from "@/components/GroupChat/ChatRoom";
import CreateRoomModal from "@/components/GroupChat/CreateRoomModal";
import styles from "./GroupChat.module.css";

export default function GroupChat() {
  const [rooms, setRooms] = useState<ChatRoomType[]>([]);
  const [activeRoomId, setActiveRoomId] = useState<string | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);

  const activeRoom = rooms.find((r) => r.id === activeRoomId) ?? null;

  const fetchRooms = useCallback(async () => {
    try {
      const list = await commands.listChatRooms();
      setRooms(list);
    } catch {
      // backend may not be ready
    }
  }, []);

  useEffect(() => {
    fetchRooms();
    const interval = setInterval(fetchRooms, 3000);
    return () => clearInterval(interval);
  }, [fetchRooms]);

  const handleCreate = useCallback(
    async (name: string, agents: Array<{ agent_type: string; name: string; role: string }>) => {
      try {
        const room = await commands.createChatRoom(name, agents);
        setActiveRoomId(room.id);
        await fetchRooms();
      } catch (err) {
        console.error("Failed to create room:", err);
      }
    },
    [fetchRooms]
  );

  const handleCloseRoom = useCallback(
    async (roomId: string) => {
      try {
        await commands.closeChatRoom(roomId);
        if (activeRoomId === roomId) {
          setActiveRoomId(null);
        }
        await fetchRooms();
      } catch (err) {
        console.error("Failed to close room:", err);
      }
    },
    [activeRoomId, fetchRooms]
  );

  return (
    <div className={styles.container}>
      <div className={styles.header}>
        <h1 className={styles.title}>Group Chat</h1>
        <button className="btn btn-primary" onClick={() => setShowCreateModal(true)}>
          <Plus size={16} />
          New Room
        </button>
      </div>

      {rooms.length === 0 ? (
        <EmptyState
          icon={Users}
          title="No chat rooms"
          description="Create a room for multi-agent collaboration"
          action={{ label: "Create Room", onClick: () => setShowCreateModal(true) }}
        />
      ) : (
        <div className={styles.workspace}>
          <div className={styles.roomList}>
            <div className={styles.roomListHeader}>
              <span className={styles.roomListTitle}>Rooms</span>
              <button
                className={styles.newRoomBtn}
                onClick={() => setShowCreateModal(true)}
              >
                <Plus size={14} />
              </button>
            </div>
            <div className={styles.roomItems}>
              {rooms.map((room) => (
                <div
                  key={room.id}
                  className={`${styles.roomItem} ${room.id === activeRoomId ? styles.roomItemActive : ""}`}
                  onClick={() => setActiveRoomId(room.id)}
                  role="button"
                  tabIndex={0}
                >
                  <div className={styles.roomName}>{room.name}</div>
                  <div className={styles.roomMeta}>
                    {room.agents.map((agent, i) => (
                      <span key={agent.name}>
                        <span
                          className={styles.roomDot}
                          style={{
                            backgroundColor: [
                              "#7c3aed", "#059669", "#d97706", "#dc2626",
                              "#2563eb", "#db2777",
                            ][i % 6],
                          }}
                        />
                        {agent.name}
                      </span>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className={styles.chatArea}>
            {activeRoom ? (
              <>
                <ChatRoom room={activeRoom} />
                <button
                  className={styles.closeRoomBtn}
                  onClick={() => handleCloseRoom(activeRoom.id)}
                >
                  Close Room
                </button>
              </>
            ) : (
              <div className={styles.chatPlaceholder}>
                Select a room to start chatting
              </div>
            )}
          </div>
        </div>
      )}

      <CreateRoomModal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        onCreate={handleCreate}
      />
    </div>
  );
}
