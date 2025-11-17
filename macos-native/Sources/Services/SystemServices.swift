/*
 * WezTerm System Services Integration
 *
 * Provides integration with macOS system services like Notification Center,
 * Spotlight, Siri Shortcuts, and more.
 */

import Foundation
import AppKit

#if canImport(UserNotifications)
import UserNotifications
#endif

/// Integration with macOS system services
@available(macOS 11.0, *)
public class SystemServices {
    /// Shared instance
    public static let shared = SystemServices()

    private init() {}

    // MARK: - Notification Center

    /// Request notification permission
    public func requestNotificationPermission() async -> Bool {
        #if canImport(UserNotifications)
        let center = UNUserNotificationCenter.current()
        do {
            let granted = try await center.requestAuthorization(options: [.alert, .sound, .badge])
            return granted
        } catch {
            return false
        }
        #else
        return false
        #endif
    }

    /// Show a notification
    public func showNotification(title: String, body: String, identifier: String = UUID().uuidString) {
        #if canImport(UserNotifications)
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = body
        content.sound = .default

        let request = UNNotificationRequest(
            identifier: identifier,
            content: content,
            trigger: nil
        )

        UNUserNotificationCenter.current().add(request) { error in
            if let error = error {
                print("Failed to show notification: \(error)")
            }
        }
        #endif
    }

    /// Show bell notification
    public func showBellNotification() {
        showNotification(
            title: "Terminal Bell",
            body: "A terminal bell was triggered",
            identifier: "wezterm-bell"
        )
    }

    /// Show process completion notification
    public func showProcessCompletionNotification(processName: String, exitCode: Int) {
        let status = exitCode == 0 ? "completed successfully" : "failed with code \(exitCode)"
        showNotification(
            title: "Process \(status)",
            body: "\(processName) has finished",
            identifier: "wezterm-process-\(UUID().uuidString)"
        )
    }

    // MARK: - Dock Badge

    /// Update dock badge with count
    public func updateDockBadge(count: Int) {
        NSApplication.shared.dockTile.badgeLabel = count > 0 ? "\(count)" : nil
    }

    /// Clear dock badge
    public func clearDockBadge() {
        NSApplication.shared.dockTile.badgeLabel = nil
    }

    /// Show progress indicator in dock
    public func showDockProgress(_ progress: Double) {
        NSApplication.shared.dockTile.contentView = DockProgressView(progress: progress)
        NSApplication.shared.dockTile.display()
    }

    /// Hide progress indicator
    public func hideDockProgress() {
        NSApplication.shared.dockTile.contentView = nil
        NSApplication.shared.dockTile.display()
    }

    // MARK: - Services Menu

    /// Register as services provider
    public func registerServices() {
        NSApplication.shared.servicesProvider = TerminalServicesProvider.shared
        NSUpdateDynamicServices()
    }

    // MARK: - Handoff

    /// Start handoff activity for terminal session
    public func startHandoffActivity(workingDirectory: String, terminalTitle: String) -> NSUserActivity {
        let activity = NSUserActivity(activityType: "com.wezterm.terminal.session")
        activity.title = terminalTitle
        activity.userInfo = [
            "workingDirectory": workingDirectory,
            "title": terminalTitle
        ]
        activity.isEligibleForHandoff = true
        activity.isEligibleForSearch = true
        activity.becomeCurrent()
        return activity
    }

    /// Continue from handoff activity
    public func continueHandoffActivity(_ activity: NSUserActivity) -> (workingDirectory: String, title: String)? {
        guard activity.activityType == "com.wezterm.terminal.session",
              let workingDir = activity.userInfo?["workingDirectory"] as? String,
              let title = activity.userInfo?["title"] as? String else {
            return nil
        }
        return (workingDir, title)
    }

    // MARK: - Quick Actions (macOS 12+)

    @available(macOS 12.0, *)
    public func setupQuickActions() {
        // Quick actions for terminal
        // This would integrate with ShortcutsProvider
    }

    // MARK: - Appearance Monitoring

    /// Monitor system appearance changes
    public func startAppearanceMonitoring(handler: @escaping (Bool) -> Void) {
        DistributedNotificationCenter.default().addObserver(
            forName: NSNotification.Name("AppleInterfaceThemeChangedNotification"),
            object: nil,
            queue: .main
        ) { _ in
            let isDark = NSApp.effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
            handler(isDark)
        }
    }

    /// Get current appearance
    public var isDarkMode: Bool {
        NSApp.effectiveAppearance.bestMatch(from: [.darkAqua, .aqua]) == .darkAqua
    }
}

// MARK: - Services Provider

@available(macOS 11.0, *)
class TerminalServicesProvider: NSObject {
    static let shared = TerminalServicesProvider()

    override private init() {
        super.init()
    }

    /// Open file/directory in WezTerm
    @objc func openInWezTerm(_ pboard: NSPasteboard, userData: String?, error: AutoreleasingUnsafeMutablePointer<NSString?>) {
        guard let items = pboard.pasteboardItems else {
            error.pointee = "No items in pasteboard" as NSString
            return
        }

        for item in items {
            if let urlString = item.string(forType: .fileURL),
               let url = URL(string: urlString) {
                let path = url.path
                // Open terminal at this path
                // WezTermBridge.shared.openNewTabWithPath(path)
                print("Opening terminal at: \(path)")
            }
        }
    }

    /// Run shell command in WezTerm
    @objc func runInWezTerm(_ pboard: NSPasteboard, userData: String?, error: AutoreleasingUnsafeMutablePointer<NSString?>) {
        guard let command = pboard.string(forType: .string) else {
            error.pointee = "No command in pasteboard" as NSString
            return
        }

        // Run command in terminal
        print("Running command: \(command)")
    }
}

// MARK: - Dock Progress View

@available(macOS 11.0, *)
class DockProgressView: NSView {
    let progress: Double

    init(progress: Double) {
        self.progress = progress.clamped(to: 0...1)
        super.init(frame: NSRect(x: 0, y: 0, width: 128, height: 128))
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func draw(_ dirtyRect: NSRect) {
        super.draw(dirtyRect)

        // Draw progress bar at bottom of dock tile
        let barHeight: CGFloat = 10
        let barY: CGFloat = 10
        let barWidth = bounds.width - 20

        // Background
        NSColor.gray.withAlphaComponent(0.3).setFill()
        let bgRect = NSRect(x: 10, y: barY, width: barWidth, height: barHeight)
        NSBezierPath(roundedRect: bgRect, xRadius: 5, yRadius: 5).fill()

        // Progress
        NSColor.systemBlue.setFill()
        let progressRect = NSRect(x: 10, y: barY, width: barWidth * CGFloat(progress), height: barHeight)
        NSBezierPath(roundedRect: progressRect, xRadius: 5, yRadius: 5).fill()
    }
}

// MARK: - Extensions

extension Comparable {
    func clamped(to range: ClosedRange<Self>) -> Self {
        return min(max(self, range.lowerBound), range.upperBound)
    }
}
