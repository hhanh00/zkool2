import 'package:flutter/material.dart';
import 'package:flutter_contacts/flutter_contacts.dart' as fc;
import 'package:gap/gap.dart';
import 'package:zkool/src/rust/api/key.dart';
import 'package:zkool/utils.dart';

class ContactCandidate {
  final String name;
  final String notes;
  final List<String> addresses;

  const ContactCandidate({
    required this.name,
    required this.notes,
    required this.addresses,
  });
}

Future<List<ContactCandidate>?> showContactPicker(
  BuildContext context, {
  required bool multiSelect,
}) async {
  // Request permission
  final status = await fc.FlutterContacts.permissions.request(
    fc.PermissionType.read,
  );
  if (status != fc.PermissionStatus.granted &&
      status != fc.PermissionStatus.limited) {
    if (context.mounted) {
      await showMessage(
        context,
        "Zkool needs access to your contacts to import Zcash addresses. "
        "Please grant contacts access in your device Settings.",
        title: "Permission Required",
      );
    }
    return null;
  }

  // Fetch all contacts with fields that may contain Zcash addresses.
  // Notes work on Android but require a special entitlement on iOS;
  // website, organization, and socialMedia are readable on both platforms.
  List<fc.Contact> deviceContacts;
  try {
    deviceContacts = await fc.FlutterContacts.getAll(
      properties: {
        fc.ContactProperty.name,
        fc.ContactProperty.note,
        fc.ContactProperty.website,
        fc.ContactProperty.organization,
        fc.ContactProperty.socialMedia,
      },
    );
  } catch (e) {
    if (context.mounted) {
      await showMessage(
        context,
        "Failed to read contacts: $e",
        title: "Error",
      );
    }
    return null;
  }

  // Filter to contacts with Zcash addresses and build candidates
  final candidates = <ContactCandidate>[];
  debugPrint('[contact_picker] Fetched ${deviceContacts.length} contacts from device');
  for (final dc in deviceContacts) {
    final addresses = <String>[];
    final noteTexts = <String>[];
    final dcName = dc.displayName ?? 'Unknown';

    // Collect all candidate text lines from various fields.
    // Notes work on Android; website/organization/socialMedia work on both
    // iOS and Android without special entitlements.
    void scanLine(String line) {
      var trimmed = line.trim();
      if (trimmed.isEmpty) return;
      // Strip zcash: URI prefix (e.g. "zcash:u1abc..." -> "u1abc...")
      const zcashPrefix = 'zcash:';
      if (trimmed.startsWith(zcashPrefix)) {
        trimmed = trimmed.substring(zcashPrefix.length).trim();
      }
      if (trimmed.isEmpty) return;
      debugPrint('[contact_picker] $dcName scanning: "$trimmed"');
      if (isValidAddress(address: trimmed)) {
        debugPrint('[contact_picker]   -> valid Zcash address');
        addresses.add(trimmed);
      } else {
        debugPrint('[contact_picker]   -> NOT a valid address');
        noteTexts.add(line.trim());
      }
    }

    // Notes (works on Android; requires entitlement on iOS)
    debugPrint('[contact_picker] $dcName notes count: ${dc.notes.length}');
    for (final note in dc.notes) {
      for (final line in note.note.split('\n')) {
        scanLine(line);
      }
    }

    // Websites (URL field — works on both platforms)
    debugPrint('[contact_picker] $dcName websites count: ${dc.websites.length}');
    for (final website in dc.websites) {
      for (final line in website.url.split('\n')) {
        scanLine(line);
      }
    }

    // Organization fields (works on both platforms)
    debugPrint('[contact_picker] $dcName orgs count: ${dc.organizations.length}');
    for (final org in dc.organizations) {
      for (final field in [org.name, org.jobTitle, org.departmentName]) {
        if (field != null) {
          for (final line in field.split('\n')) {
            scanLine(line);
          }
        }
      }
    }

    // Social media usernames (works on both platforms)
    debugPrint('[contact_picker] $dcName social count: ${dc.socialMedias.length}');
    for (final sm in dc.socialMedias) {
      for (final line in sm.username.split('\n')) {
        scanLine(line);
      }
    }

    // Only include contacts that have at least one Zcash address
    if (addresses.isEmpty) {
      debugPrint('[contact_picker] $dcName -> SKIPPED (no Zcash address found)');
      continue;
    }

    debugPrint('[contact_picker] "$dcName" -> CANDIDATE (${addresses.length} address(es))');
    candidates.add(ContactCandidate(
      name: dcName,
      notes: noteTexts.join('\n'),
      addresses: addresses,
    ));
  }

  if (!context.mounted) return null;

  if (candidates.isEmpty) {
    await showMessage(
      context,
      "No contacts with Zcash addresses were found. "
      "Add a Zcash address to a contact's Notes (Android), "
      "or URL / Company fields (iOS & Android) to import it.",
      title: "No Addresses Found",
    );
    return null;
  }

  // Show the picker dialog
  final result = await Navigator.of(context).push<Iterable<ContactCandidate>>(
    MaterialPageRoute(
      builder: (_) => _ContactPickerDialog(
        candidates: candidates,
        multiSelect: multiSelect,
      ),
    ),
  );
  return result?.toList();
}

class _ContactPickerDialog extends StatefulWidget {
  final List<ContactCandidate> candidates;
  final bool multiSelect;

  const _ContactPickerDialog({
    required this.candidates,
    required this.multiSelect,
  });

  @override
  State<_ContactPickerDialog> createState() => _ContactPickerDialogState();
}

class _ContactPickerDialogState extends State<_ContactPickerDialog> {
  final _searchController = TextEditingController();
  final _selected = <ContactCandidate>{};
  String _query = '';

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  List<ContactCandidate> get _filtered {
    if (_query.isEmpty) return widget.candidates;
    final q = _query.toLowerCase();
    return widget.candidates
        .where((c) => c.name.toLowerCase().contains(q))
        .toList();
  }

  @override
  Widget build(BuildContext context) {
    final filtered = _filtered;

    return Scaffold(
      appBar: AppBar(
        title: Text(widget.multiSelect ? "Import Contacts" : "Select Contact"),
        leading: IconButton(
          icon: const Icon(Icons.close),
          onPressed: () => Navigator.of(context).pop(null),
        ),
      ),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(12),
            child: TextField(
              controller: _searchController,
              decoration: const InputDecoration(
                hintText: "Search contacts...",
                prefixIcon: Icon(Icons.search),
                border: OutlineInputBorder(),
              ),
              onChanged: (v) => setState(() => _query = v),
            ),
          ),
          Expanded(
            child: filtered.isEmpty
                ? Center(
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(
                          _query.isEmpty
                              ? Icons.contact_phone
                              : Icons.search_off,
                          size: 64,
                          color: Theme.of(context)
                              .colorScheme
                              .onSurface
                              .withAlpha(100),
                        ),
                        Gap(16),
                        Text(
                          _query.isEmpty
                              ? "No contacts available"
                              : "No contacts matching \"$_query\"",
                          style: Theme.of(context).textTheme.bodyLarge,
                        ),
                      ],
                    ),
                  )
                : ListView.builder(
                    itemCount: filtered.length,
                    itemBuilder: (context, index) {
                      final candidate = filtered[index];
                      final isSelected = _selected.contains(candidate);
                      return ListTile(
                        leading: widget.multiSelect
                            ? Checkbox(
                                value: isSelected,
                                onChanged: (_) {
                                  setState(() {
                                    if (isSelected) {
                                      _selected.remove(candidate);
                                    } else {
                                      _selected.add(candidate);
                                    }
                                  });
                                },
                              )
                            : const Icon(Icons.person),
                        title: Text(candidate.name),
                        subtitle: Text(
                          candidate.addresses.length == 1
                              ? "1 Zcash address found"
                              : "${candidate.addresses.length} Zcash addresses found",
                        ),
                        onTap: () {
                          if (widget.multiSelect) {
                            setState(() {
                              if (isSelected) {
                                _selected.remove(candidate);
                              } else {
                                _selected.add(candidate);
                              }
                            });
                          } else {
                            Navigator.of(context)
                                .pop([candidate]);
                          }
                        },
                      );
                    },
                  ),
          ),
        ],
      ),
      bottomNavigationBar: widget.multiSelect && _selected.isNotEmpty
          ? SafeArea(
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: FilledButton.icon(
                  icon: const Icon(Icons.person_add),
                  label: Text("Import ${_selected.length} contact(s)"),
                  onPressed: () {
                    Navigator.of(context).pop(_selected.toList());
                  },
                ),
              ),
            )
          : null,
    );
  }
}
