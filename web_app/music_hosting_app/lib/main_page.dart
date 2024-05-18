import 'package:flutter/material.dart';
import 'package:http/http.dart' as http;

import 'shared_state.dart';
import 'signup_page.dart';
import 'login_page.dart';
import 'upload_track.dart';
import 'search_results_page.dart';
import 'music_player.dart';

class MainPage extends StatefulWidget {
  const MainPage({super.key});

  @override
  _MainPageState createState() => _MainPageState();
}

class _MainPageState extends State<MainPage> {
  bool _isTokenValid = false;
  String username = "USERNAME INITIAL VALUE";
  final TextEditingController _searchController = TextEditingController();

  @override
  void initState() {
    super.initState();
    _checkTokenValidity();
  }

  Future<void> _checkTokenValidity() async {
    username = await getUsername() ?? "FAILED TO GET USERNAME";
    setState(() {
      _isTokenValid = false;
    });
    try {
      final token = (await getToken()) ?? "";
      if (token != "") {
        final response = await http.get(
            Uri.parse('http://localhost:3000/check_token'),
            headers: {'Authorization': token});
        if (response.statusCode == 200) {
          setState(() {
            _isTokenValid = true;
          });
        }
      }
    } catch (e) {}
  }

  Future<void> _logout() async {
    final token = (await getToken())!;
    storeToken(username, "");

    final response = await http.post(
      Uri.parse('http://localhost:3000/logout'),
      headers: {'authorization': token},
    );

    Navigator.pushReplacement(
        context,
        MaterialPageRoute(
          builder: (context) => const MainPage(),
        ));
  }

  @override
  Widget build(BuildContext context) {
    return _isTokenValid
        ? _buildWelcomePage(context)
        : _buildLoginSignupPage(context);
  }

  Widget _buildLoginSignupPage(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Main Page'),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            ElevatedButton(
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(builder: (context) => const LoginPage()),
                );
              },
              child: const Text('Log in'),
            ),
            const SizedBox(height: 16),
            ElevatedButton(
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(builder: (context) => const SignupPage()),
                );
              },
              child: const Text('Sign up'),
            ),
          ],
        ),
      ),
    );
  }

  void _search(BuildContext context) {
    String query = _searchController.text;
    Navigator.push(
      context,
      MaterialPageRoute(
        builder: (context) => SearchResultsPage(query: query),
      ),
    );
  }

  Widget _buildWelcomePage(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Hello, $username!'),
        actions: [
          Padding(
            padding: const EdgeInsets.only(right: 10.0),
            child: Align(
              alignment: Alignment.centerRight,
              child: ElevatedButton(
                onPressed: _logout,
                child: const Text('Log out'),
              ),
            ),
          ),
        ],
        bottom: PreferredSize(
          preferredSize: const Size.fromHeight(70.0),
          child: Padding(
            padding: const EdgeInsets.all(8.0),
            child: TextField(
              controller: _searchController,
              decoration: InputDecoration(
                hintText: 'Search...',
                border:
                    OutlineInputBorder(borderRadius: BorderRadius.circular(8)),
                suffixIcon: IconButton(
                  icon: const Icon(Icons.search),
                  onPressed: () => _search(context),
                ),
              ),
              onSubmitted: (value) => _search(context),
            ),
          ),
        ),
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            ElevatedButton(
              onPressed: () {
                Navigator.push(
                  context,
                  MaterialPageRoute(builder: (context) => const UploadTrackPage()),
                );
              },
              child: const Text('Upload track'),
            ),
          ],
        ),
      ),
      bottomNavigationBar: const MusicPlayer(),
    );
  }
}
