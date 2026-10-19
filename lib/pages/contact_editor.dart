import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_form_builder/flutter_form_builder.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:gap/gap.dart';
import 'package:go_router/go_router.dart';
import 'package:file_picker/file_picker.dart';
import 'package:zkool/src/rust/api/contacts.dart';
import 'package:zkool/src/rust/api/openalias.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';
import 'package:zkool/validators.dart';
import 'package:zkool/widgets/contact_picker.dart';
import 'package:zkool/widgets/scanner.dart';

class ContactEditPage extends ConsumerStatefulWidget {
  final Contact? contact;

  const ContactEditPage({super.key, this.contact});

  @override
  ConsumerState<ContactEditPage> createState() => ContactEditPageState();
}

/// One address row of the editor. The id is stable for the row's lifetime
/// and names its form field (`address_<id>`), so removing a row in the middle
/// does not shift the other rows' state. A row whose value is set from code
/// gets a new id, so its field is rebuilt with the new initial value.
class _AddressRow {
  final int id;
  final String initial;
  const _AddressRow(this.id, this.initial);
}

class ContactEditPageState extends ConsumerState<ContactEditPage> {
  final _formKey = GlobalKey<FormBuilderState>();
  late var c = coinContext.coin;
  var _nextRowId = 0;
  var _rows = <_AddressRow>[];
  // Set when an address row was removed or filled from code: those changes
  // do not show in the form's own dirty state.
  var _rowsChanged = false;

  bool get isEditing => widget.contact != null;

  @override
  void initState() {
    super.initState();
    _rows = _rowsFor(widget.contact?.addresses ?? const []);
  }

  _AddressRow _newRow(String initial) => _AddressRow(_nextRowId++, initial);

  /// Rows for the given addresses; at least one, empty, row.
  List<_AddressRow> _rowsFor(List<String> addresses) =>
      addresses.isEmpty ? [_newRow('')] : addresses.map(_newRow).toList();

  /// The current text of each address row, as typed.
  List<String> _currentAddresses() => _rows
      .map((row) => (_formKey.currentState?.fields['address_${row.id}']?.value as String?) ?? row.initial)
      .toList();

  /// Reads the saved form: name, notes, and the non-empty addresses of the
  /// rows that still exist.
  (String, String, List<String>) _formValues() {
    final values = _formKey.currentState!.value;
    final name = (values['name'] as String?)?.trim() ?? '';
    final notes = (values['notes'] as String?)?.trim() ?? '';
    final addresses = <String>[];
    for (final row in _rows) {
      final addr = (values['address_${row.id}'] as String?)?.trim() ?? '';
      if (addr.isNotEmpty) addresses.add(addr);
    }
    return (name, notes, addresses);
  }

  /// Replaces the whole form: name, notes and addresses.
  void _load({required String name, required String notes, required List<String> addresses}) {
    setState(() {
      final form = _formKey.currentState!;
      form.fields['name']!.didChange(name);
      form.fields['notes']!.didChange(notes);
      _rows = _rowsFor(addresses);
      _rowsChanged = true;
    });
  }

  /// Adds addresses to the form: empty rows are filled first, then rows are
  /// appended. Rows with text keep it.
  void _addAddresses(Iterable<String> addresses) {
    final current = _currentAddresses();
    final rows = List<_AddressRow>.from(_rows);
    for (final address in addresses) {
      final emptyIndex = current.indexWhere((a) => a.trim().isEmpty);
      if (emptyIndex != -1) {
        rows[emptyIndex] = _newRow(address);
        current[emptyIndex] = address;
      } else {
        rows.add(_newRow(address));
        current.add(address);
      }
    }
    setState(() {
      _rows = rows;
      _rowsChanged = true;
    });
  }

  @override
  Widget build(BuildContext context) {
    return PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, _) {
        if (didPop) return;
        // If form is unchanged, just pop without confirmation
        if (_isFormClean()) {
          GoRouter.of(context).pop();
          return;
        }
        onSaveAndPop();
      },
      child: Scaffold(
        appBar: AppBar(
          title: Text(isEditing ? "Edit Contact" : "New Contact"),
          actions: [
            IconButton(onPressed: onImport, tooltip: "Import from vCard", icon: Icon(Icons.download)),
            IconButton(onPressed: onImportFromContacts, tooltip: "Import from Contacts", icon: Icon(Icons.contacts)),
            IconButton(onPressed: onExport, tooltip: "Export as vCard", icon: Icon(Icons.upload_file)),
            if (isEditing) IconButton(onPressed: onDelete, tooltip: "Delete contact", icon: Icon(Icons.delete)),
          ],
        ),
        body: SingleChildScrollView(
          padding: const EdgeInsets.symmetric(horizontal: 12),
          child: FormBuilder(
            key: _formKey,
            child: Column(
              children: [
                Gap(16),
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(20),
                    child: Column(
                      children: [
                        FormBuilderTextField(
                          name: "name",
                          decoration: const InputDecoration(
                            labelText: "Name",
                            prefixIcon: Icon(Icons.person_outline),
                          ),
                          initialValue: widget.contact?.name ?? '',
                          validator: (v) => (v == null || v.trim().isEmpty) ? 'Name is required' : null,
                        ),
                      ],
                    ),
                  ),
                ),
                Gap(12),
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(20),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            Text("Addresses", style: Theme.of(context).textTheme.titleMedium),
                            IconButton(
                              icon: Icon(Icons.dns_outlined),
                              tooltip: "Import from OpenAlias",
                              onPressed: () => onImportOpenalias(),
                            ),
                          ],
                        ),
                        Gap(8),
                        ..._buildAddressFields(),
                      ],
                    ),
                  ),
                ),
                Gap(12),
                Card(
                  child: Padding(
                    padding: const EdgeInsets.all(20),
                    child: Column(
                      children: [
                        FormBuilderTextField(
                          name: "notes",
                          decoration: const InputDecoration(
                            labelText: "Notes",
                            prefixIcon: Icon(Icons.notes),
                          ),
                          initialValue: widget.contact?.notes ?? '',
                          maxLines: 3,
                        ),
                      ],
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  List<Widget> _buildAddressFields() {
    final widgets = <Widget>[];
    for (var i = 0; i < _rows.length; i++) {
      final row = _rows[i];
      widgets.add(
        Row(
          key: ValueKey(row.id),
          children: [
            Expanded(
              child: FormBuilderTextField(
                name: "address_${row.id}",
                decoration: InputDecoration(
                  labelText: "Address ${i + 1}",
                  prefixIcon: Icon(Icons.qr_code),
                ),
                initialValue: row.initial,
                validator: (v) {
                  if (v == null || v.trim().isEmpty) return 'Address is required';
                  if (validAddress(v) != null) return 'Invalid address';
                  return null;
                },
              ),
            ),
            IconButton(
              icon: Icon(Icons.qr_code_scanner, color: Colors.blue),
              onPressed: () => onScanAddress(i),
              tooltip: "Scan QR code",
            ),
            if (i > 0)
              IconButton(
                icon: Icon(Icons.remove_circle_outline, color: Colors.red),
                onPressed: () => setState(() {
                  _rows.removeAt(i);
                  _rowsChanged = true;
                }),
                tooltip: "Remove address",
              ),
          ],
        ),
      );
      widgets.add(Gap(8));
    }
    widgets.add(
      Center(
        child: TextButton.icon(
          icon: Icon(Icons.add),
          label: Text("Add Address"),
          onPressed: () => setState(() => _rows.add(_newRow(''))),
        ),
      ),
    );
    return widgets;
  }

  bool _isFormClean() => !_rowsChanged && !(_formKey.currentState?.isDirty ?? false);

  void onScanAddress(int index) async {
    final scanned = await showScanner(context, validator: validAddress);
    if (scanned != null && index < _rows.length) {
      setState(() {
        _rows[index] = _newRow(scanned);
        _rowsChanged = true;
      });
    }
  }

  void onImportOpenalias() async {
    print("Importing OpenAlias");
    final alias = await inputText(context, title: "Enter OpenAlias name");
    if (alias == null || alias.trim().isEmpty) return;

    try {
      final result = await resolveOpenalias(alias: alias.trim(), c: c);
      if (result.recipients.isEmpty) {
        showMessage(context, "No Zcash addresses were found for '$alias'.", title: "No address found");
        return;
      }
      if (result.dnssecStatus != 'verified') {
        final (title, message) = switch (result.dnssecStatus) {
          'unsigned' => (
              'DNSSEC not available',
              'The domain for $alias does not support DNSSEC. '
                  'The response could not be cryptographically verified. '
                  'Proceed anyway?',
            ),
          'bogus' => (
              'DNSSEC validation failed',
              'DNSSEC validation FAILED for $alias! '
                  'The response may have been tampered with. '
                  'Proceed at your own risk?',
            ),
          _ => ('Untrusted response', 'Proceed anyway?'),
        };
        final confirmed = await confirmDialog(
          context,
          title: title,
          message: message,
        );
        if (!confirmed) return;
      }
      _addAddresses(result.recipients.map((r) => r.address));
    } on AnyhowException catch (e) {
      showMessage(context, e.message, title: "OpenAlias Error");
    } catch (e) {
      showMessage(context, e.toString(), title: "OpenAlias Error");
    }
  }

  void onImportFromContacts() async {
    final candidates = await showContactPicker(context, multiSelect: false);
    if (candidates == null || candidates.isEmpty) return;
    final candidate = candidates.first;
    _load(name: candidate.name, notes: candidate.notes, addresses: candidate.addresses);
  }

  void onSaveAndPop() async {
    final form = _formKey.currentState!;
    form.save();
    final (name, notes, addresses) = _formValues();

    // Empty form: just pop without saving
    if (name.isEmpty && addresses.isEmpty) {
      if (mounted) GoRouter.of(context).pop();
      return;
    }

    // Validate; if invalid, block pop
    if (!form.validate()) return;

    if (isEditing) {
      await updateContact(id: widget.contact!.id, name: name, addresses: addresses, notes: notes, c: c);
    } else {
      await createContact(name: name, addresses: addresses, notes: notes, c: c);
    }

    ref.invalidate(getContactsProvider);
    if (mounted) GoRouter.of(context).pop();
  }

  void onDelete() async {
    final confirmed = await confirmDialog(
      context,
      title: "Delete contact?",
      message: "'${widget.contact!.name}' will be permanently deleted.",
    );
    if (confirmed) {
      await deleteContacts(ids: [widget.contact!.id], c: c);
      ref.invalidate(getContactsProvider);
      if (mounted) GoRouter.of(context).pop();
    }
  }

  void onImport() async {
    final result = await FilePicker.platform.pickFiles(
      type: FileType.any,
      allowMultiple: false,
    );
    if (result == null || result.files.single.path == null) return;
    final data = await File(result.files.single.path!).readAsString();
    final contacts = parseContactsVcard(vcardData: data);
    if (!mounted) return;
    if (contacts.isEmpty) {
      await showMessage(context, "The file has no contact with a Zcash address.", title: "Import from vCard");
      return;
    }
    final contact = contacts.first;
    _load(name: contact.name, notes: contact.notes, addresses: contact.addresses);
    if (contacts.length > 1) {
      await showMessage(
        context,
        "The file has ${contacts.length} contacts; only the first one was loaded. "
        "To import them all, use Import from vCard on the contacts list.",
        title: "Import from vCard",
      );
    }
  }

  void onExport() async {
    final form = _formKey.currentState!;
    if (!form.saveAndValidate()) return;
    final (name, notes, addresses) = _formValues();
    final vcard = exportContactVcard(name: name, addresses: addresses, notes: notes);

    await saveFile(
      title: 'Export Contact',
      fileName: '${name.replaceAll(RegExp(r'[^a-zA-Z0-9]'), '_')}.vcf',
      data: Uint8List.fromList(vcard.codeUnits),
    );
  }
}
