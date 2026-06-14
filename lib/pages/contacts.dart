import 'dart:io';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:file_picker/file_picker.dart';
import 'package:zkool/src/rust/api/contacts.dart';
import 'package:zkool/store.dart';
import 'package:zkool/utils.dart';

class ContactListPage extends ConsumerStatefulWidget {
  const ContactListPage({super.key});

  @override
  ConsumerState<ContactListPage> createState() => ContactListPageState();
}

class ContactListPageState extends ConsumerState<ContactListPage> {
  late final c = coinContext.coin;
  List<Contact> contacts = [];

  @override
  void initState() {
    super.initState();
    Future(refresh);
  }

  Future<void> refresh() async {
    final contacts = await ref.read(getContactsProvider.future);
    if (mounted) {
      setState(() => this.contacts = contacts);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text("Contacts"),
        actions: [
          IconButton(onPressed: onImportVcard, tooltip: "Import from vCard", icon: Icon(Icons.download)),
          IconButton(onPressed: onExportVcard, tooltip: "Export as vCard", icon: Icon(Icons.upload_file)),
          IconButton(onPressed: onNew, tooltip: "New contact", icon: Icon(Icons.add)),
        ],
      ),
      body: ListView.builder(
        itemBuilder: (BuildContext context, int index) {
          final contact = contacts[index];
          return ListTile(
            title: Text(contact.name),
            subtitle: Text(contact.addresses.isNotEmpty ? contact.addresses.first : 'No addresses'),
            onTap: () => onEdit(contact),
          );
        },
        itemCount: contacts.length,
      ),
    );
  }

  void onNew() async {
    await GoRouter.of(context).push('/contact/edit');
    await refresh();
  }

  void onEdit(Contact contact) async {
    await GoRouter.of(context).push('/contact/edit', extra: contact);
    await refresh();
  }

  void onImportVcard() async {
    final result = await FilePicker.platform.pickFiles(
      type: FileType.any,
      allowMultiple: false,
    );
    if (result != null && result.files.single.path != null) {
      final path = result.files.single.path!;
      final data = await File(path).readAsString();
      await importContactsVcard(vcardData: data, c: c);
      await refresh();
      ref.invalidate(getContactsProvider);
    }
  }

  void onExportVcard() async {
    final data = await exportContactsVcard(c: c);
    await saveFile(
      title: 'Export Contacts',
      fileName: 'contacts.vcf',
      data: Uint8List.fromList(data.codeUnits),
    );
  }
}
